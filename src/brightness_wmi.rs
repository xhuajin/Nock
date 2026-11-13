//! Native WMI brightness fallback, called only on short-lived worker threads.

use windows::core::{w, Error, Result, BSTR, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_TIMEOUT, E_FAIL, RPC_E_TOO_LATE};
use windows::Win32::System::Com::*;
use windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::System::Wmi::*;

struct Apartment;

impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct Wmi {
    // Release the proxy before uninitializing this thread's COM apartment.
    services: IWbemServices,
    _apartment: Apartment,
}

impl Wmi {
    fn connect() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            let apartment = Apartment;
            // Other Windows integrations may have initialized process security.
            // Set impersonation explicitly on our proxy in either case.
            if let Err(e) = CoInitializeSecurity(
                None,
                -1,
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            ) {
                if e.code() != RPC_E_TOO_LATE {
                    return Err(e);
                }
            }
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
            let empty = BSTR::new();
            let services = locator.ConnectServer(
                &BSTR::from("ROOT\\WMI"),
                &empty,
                &empty,
                &empty,
                WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
                &empty,
                None,
            )?;
            CoSetProxyBlanket(
                &services,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHZ_NONE,
                PCWSTR::null(),
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )?;
            Ok(Self {
                services,
                _apartment: apartment,
            })
        }
    }

    fn query(&self, query: &str) -> Result<IEnumWbemClassObject> {
        unsafe {
            self.services.ExecQuery(
                &BSTR::from("WQL"),
                &BSTR::from(query),
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
                None,
            )
        }
    }
}

fn next(objects: &IEnumWbemClassObject) -> Result<Option<IWbemClassObject>> {
    let mut row = [None];
    let mut returned = 0;
    let status = unsafe { objects.Next(2_000, &mut row, &mut returned) };
    if status == HRESULT(WBEM_S_TIMEDOUT.0) {
        return Err(Error::from_hresult(HRESULT::from_win32(ERROR_TIMEOUT.0)));
    }
    status.ok()?;
    Ok(row[0].take())
}

fn property(object: &IWbemClassObject, name: PCWSTR) -> Result<VARIANT> {
    let mut value = VARIANT::default();
    unsafe { object.Get(name, 0, &mut value, None, None)? };
    Ok(value)
}

pub fn get() -> Result<Option<u32>> {
    let wmi = Wmi::connect()?;
    let monitors =
        wmi.query("SELECT CurrentBrightness FROM WmiMonitorBrightness WHERE Active = TRUE")?;
    let Some(monitor) = next(&monitors)? else {
        return Ok(None);
    };
    let value = u32::try_from(&property(&monitor, w!("CurrentBrightness"))?)?;
    if value > 100 {
        return Err(Error::from_hresult(E_FAIL));
    }
    Ok(Some(value))
}

pub fn set(percent: u32) -> Result<()> {
    let wmi = Wmi::connect()?;
    let monitors = wmi.query("SELECT * FROM WmiMonitorBrightnessMethods WHERE Active = TRUE")?;
    let mut class = None;
    unsafe {
        wmi.services
            .GetObject(
                &BSTR::from("WmiMonitorBrightnessMethods"),
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut class),
                None,
            )
            .map_err(|e| Error::new(e.code(), format!("brightness method class: {e}")))?;
        let mut signature = None;
        class
            .ok_or_else(|| Error::new(E_FAIL, "brightness method class missing"))?
            .GetMethod(
                w!("WmiSetBrightness"),
                0,
                &mut signature,
                std::ptr::null_mut(),
            )
            .map_err(|e| Error::new(e.code(), format!("brightness method signature: {e}")))?;
        let input = signature
            .ok_or_else(|| Error::new(E_FAIL, "brightness method signature missing"))?
            .SpawnInstance(0)
            .map_err(|e| Error::new(e.code(), format!("brightness method parameters: {e}")))?;
        // CIM uint32 uses VT_I4; CIM uint8 uses VT_UI1.
        input
            .Put(w!("Timeout"), 0, &VARIANT::from(1i32), 0)
            .map_err(|e| Error::new(e.code(), format!("WmiSetBrightness Timeout: {e}")))?;
        input
            .Put(
                w!("Brightness"),
                0,
                &VARIANT::from(percent.min(100) as u8),
                0,
            )
            .map_err(|e| Error::new(e.code(), format!("WmiSetBrightness Brightness: {e}")))?;
        while let Some(monitor) = next(&monitors)? {
            let path = BSTR::try_from(&property(&monitor, w!("__PATH"))?)?;
            let mut output = None;
            wmi.services
                .ExecMethod(
                    &path,
                    &BSTR::from("WmiSetBrightness"),
                    WBEM_FLAG_RETURN_WBEM_COMPLETE,
                    None,
                    &input,
                    Some(&mut output),
                    None,
                )
                .map_err(|e| Error::new(e.code(), format!("WmiSetBrightness ExecMethod: {e}")))?;
            // Some monitor providers complete successfully without an output
            // object. Check the method's return code when one is supplied.
            if let Some(output) = output {
                let result = u32::try_from(&property(&output, w!("ReturnValue"))?)?;
                if result != 0 {
                    return Err(Error::new(
                        E_FAIL,
                        format!("WmiSetBrightness returned {result}"),
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a local Windows WMI brightness provider"]
    fn read_and_reapply_current_brightness() {
        let current = get().expect("native WMI brightness query failed");
        println!("WMI brightness: {current:?}");
        if let Some(percent) = current {
            set(percent).expect("native WMI brightness method failed");
            assert_eq!(get().unwrap(), Some(percent));
        }
    }
}
