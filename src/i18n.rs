//! Tiny i18n: the UI language plus an English → Chinese lookup.
//!
//! `tr` is called from the draw-text hooks (`gfx_win::Gfx::text`,
//! `gfx_linux::text`) and from a handful of format sites, so ordinary string
//! literals translate with no call-site changes. Strings that are also state
//! keys (alert names like "Battery low") stay English in state and are
//! translated only here, at display time. Game text, scene dialogue, buddy
//! quotes and icon glyphs are not in the table and pass through unchanged.

use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(EN);

pub const EN: u8 = 0;
pub const ZH: u8 = 1;

pub fn lang() -> u8 {
    LANG.load(Ordering::Relaxed)
}

pub fn is_zh() -> bool {
    lang() == ZH
}

pub fn set_lang(l: u8) {
    LANG.store(if l == ZH { ZH } else { EN }, Ordering::Relaxed);
}

/// First-run default: Chinese when the system UI language is Chinese.
pub fn detect_zh() -> bool {
    #[cfg(windows)]
    {
        // GetUserDefaultUILanguage; primary language id 0x04 = Chinese
        (unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff) == 0x04
    }
    #[cfg(not(windows))]
    {
        std::env::var("LANG")
            .map(|l| l.to_ascii_lowercase().starts_with("zh"))
            .unwrap_or(false)
    }
}

/// Translate a UI literal (or `{name}` template) into the active language.
/// Unknown strings pass through unchanged.
pub fn tr(s: &str) -> &str {
    if !is_zh() {
        return s;
    }
    match s {
        // ------------------------------------------------------------ tabs
        "Media" => "媒体",
        "Controls" => "控制",
        "Today" => "今天",
        "Stats" => "统计",
        "Clipboard" => "剪贴板",
        "Shelf" => "架子",
        "Apps" => "应用",
        "Games" => "游戏",
        "Power" => "电源",
        "‹ Games" => "‹ 游戏",

        // ------------------------------------------------------- media tab
        "Nothing playing" => "未在播放",
        "Start music or a video and it shows up here" => "开始播放音乐或视频后会显示在这里",
        "Unknown title" => "未知标题",
        "Loading next track…" => "正在加载下一曲…",
        "Loading previous track…" => "正在加载上一曲…",
        "Muted" => "已静音",

        // ----------------------------------------------------- controls tab
        "Screenshot" => "截图",
        "Snip a region" => "区域截图",
        "Display" => "显示",
        "Resolution, scale, night light" => "分辨率、缩放、夜间模式",
        "Sound" => "声音",
        "Output, devices, volume mixer" => "输出、设备、音量合成器",
        "Network" => "网络",
        "Wi-Fi, Ethernet, VPN" => "Wi-Fi、以太网、VPN",
        "Power & battery" => "电源和电池",
        "Sleep, plan, battery saver" => "睡眠、电源计划、省电模式",
        "All settings" => "所有设置",
        "Open Windows Settings" => "打开 Windows 设置",
        "Lock" => "锁定",
        "Sleep" => "睡眠",
        "Restart" => "重启",
        "Shut down" => "关机",
        "Click again to confirm" => "再点一次确认",
        "Asks to confirm" => "需再次确认",

        // --------------------------------------------------------- stats tab
        "Memory" => "内存",
        "Battery" => "电池",
        "Charging" => "正在充电",
        "Plugged in" => "已接通电源",
        "On battery" => "使用电池",
        "Uptime" => "开机时长",
        "No battery" => "无电池",
        "{d}d {hh}h {m}m" => "{d}天 {hh}小时 {m}分",
        "{hh}h {m}m" => "{hh}小时 {m}分",

        // ----------------------------------------------------- clipboard tab
        "Search" => "搜索",
        "Copy something and it will appear here." => "复制的内容会显示在这里。",
        "No matches." => "没有匹配项。",
        "Clear" => "清除",
        "Clear all" => "全部清除",
        "Click an item to paste it. Click the star to pin it as a saved snippet." => {
            "点击条目即可粘贴。点击星号可固定为常用片段。"
        }

        // -------------------------------------------------------- shelf tab
        "Done" => "完成",
        "Drop to add to the shelf" => "拖入即可放到架子",
        "Drop files here to keep them handy" => "把文件拖到这里，随手取用",
        "Drag them back out into any app. Click one to open it." => "可拖回任意应用。单击打开。",

        // ---------------------------------------------------------- apps tab
        "Add an app" => "添加应用",
        "Add app" => "添加应用",
        "Search apps" => "搜索应用",
        "Updating…" => "正在更新…",
        "Windows is taking longer than usual to list your apps." => "Windows 列出应用的时间比平时长。",
        "Try again" => "重试",
        "Finding your apps…" => "正在查找应用…",
        "No apps found" => "未找到应用",
        "No apps match that search" => "没有匹配的应用",
        "Pin to panel" => "固定到面板",
        "✓ Pinned" => "✓ 已固定",

        // ------------------------------------------------------ today / timer
        "Focus timer" => "专注计时",
        "Focus · {short}" => "专注 · {short}",
        "Today: {f} min focus · {n} done" => "今天：专注 {f} 分钟 · 完成 {n} 项",
        "Enter to start · Esc to cancel" => "回车开始 · Esc 取消",
        "Enter to add · Esc to cancel" => "回车添加 · Esc 取消",
        "Pause" => "暂停",
        "Start" => "开始",
        "To-do" => "待办",
        "Add task" => "添加待办",
        "Break" => "休息",
        "{}│ min" => "{}│ 分钟",

        // -------------------------------------------- smart pill and alerts
        "Meeting" => "会议",
        "Battery low" => "电量低",
        "Battery low {t}" => "电量低 {t}",
        "Take a break" => "休息一下",
        "Look away" => "远眺放松",
        "Drink water" => "喝口水",
        "Time's up" => "时间到",
        "Back to focus" => "回到专注",
        "Break time" => "休息时间",
        "Mic on" => "麦克风开启",
        "Mic {}m" => "麦克风 {}m",
        "Unknown" => "未知",
        "now" => "现在",
        "{mins}m" => "{mins} 分钟",
        "{}m" => "{} 分钟",
        "{}h{:02}m" => "{}小时{:02}分",

        // ------------------------------------------------------------ popups
        "Wi-Fi" => "Wi-Fi",
        "Bluetooth" => "蓝牙",
        "Not available" => "不可用",
        "Off" => "关闭",
        "Not connected" => "未连接",
        "Connected" => "已连接",
        "Connect" => "连接",
        "Disconnect" => "断开",
        "Connecting" => "正在连接",
        "Connecting…" => "正在连接…",
        "Connecting to" => "正在连接到",
        "Connecting to {}…" => "正在连接到 {name}…",
        "Disconnected" => "已断开连接",
        "Disconnecting" => "正在断开",
        "Disconnecting…" => "正在断开…",
        "Connected to {name}" => "已连接到 {name}",
        "Connected to {}" => "已连接到 {name}",
        "Disconnected from {name}" => "已与 {name} 断开连接",
        "Couldn't connect to {name}" => "无法连接到 {name}",
        "Couldn't disconnect {name}" => "无法断开 {name}",
        "Signal {}%" => "信号强度 {}%",
        "No devices connected" => "未连接设备",
        "No paired devices" => "没有已配对的设备",
        "Wi-Fi isn't available on this PC" => "此电脑不支持 Wi-Fi",
        "Bluetooth isn't available on this PC" => "此电脑不支持蓝牙",
        "Looking for networks…" => "正在查找网络…",
        "Password for {}" => "{name} 的密码",
        "Enter password" => "输入密码",
        "Hide" => "隐藏",
        "Show" => "显示",
        "Cancel" => "取消",
        "Join" => "连接",
        "Password must be at least 8 characters" => "密码至少需要 8 个字符",
        "Plugged in, not charging" => "已接通电源，未在充电",
        "No battery detected" => "未检测到电池",
        "Brightness {}%" => "亮度 {}%",
        "Drag to set brightness" => "拖动调节亮度",
        " · Battery saver on" => " · 省电模式已开",
        "{st} · {h} h {m} min left" => "{st} · 剩余 {h} 小时 {m} 分钟",
        "{st} · {m} min left" => "{st} · 剩余 {m} 分钟",
        "Wi-Fi settings…" => "Wi-Fi 设置…",
        "Bluetooth settings…" => "蓝牙设置…",
        "Battery settings…" => "电池设置…",
        "Date & time settings…" => "日期和时间设置…",

        // -------------------------------------------------------- top bar
        "AM" => "上午",
        "PM" => "下午",
        "It's Friday!" => "今天是周五！",
        "{d}d {h}h to Friday" => "距周五还有 {d}天 {h}小时",
        "Desktop" => "桌面",
        "File Explorer" => "文件资源管理器",
        "Terminal" => "终端",
        "App" => "应用",

        // ------------------------------------------------------------- days
        "Mon" => "周一",
        "Tue" => "周二",
        "Wed" => "周三",
        "Thu" => "周四",
        "Fri" => "周五",
        "Sat" => "周六",
        "Sun" => "周日",
        "Monday" => "星期一",
        "Tuesday" => "星期二",
        "Wednesday" => "星期三",
        "Thursday" => "星期四",
        "Friday" => "星期五",
        "Saturday" => "星期六",
        "Sunday" => "星期日",

        // ----------------------------------------------------------- months
        "January" => "1月",
        "February" => "2月",
        "March" => "3月",
        "April" => "4月",
        "May" => "5月",
        "June" => "6月",
        "July" => "7月",
        "August" => "8月",
        "September" => "9月",
        "October" => "10月",
        "November" => "11月",
        "December" => "12月",
        "Jan" => "1月",
        "Feb" => "2月",
        "Mar" => "3月",
        "Apr" => "4月",
        "Jun" => "6月",
        "Jul" => "7月",
        "Aug" => "8月",
        "Sep" => "9月",
        "Oct" => "10月",
        "Nov" => "11月",
        "Dec" => "12月",

        // -------------------------------------------------------- tray menu
        "Show notch  (Ctrl+Alt+H)" => "显示刘海  (Ctrl+Alt+H)",
        "Hide notch  (Ctrl+Alt+H)" => "隐藏刘海  (Ctrl+Alt+H)",
        "24-hour clock" => "24 小时制",
        "Top bar" => "顶栏",
        "Hide in full-screen apps" => "全屏应用中自动隐藏",
        "Auto-hide (show when pointer hits the top edge)" => "自动隐藏（指针移到屏幕顶缘时显示）",
        "Fade out when a window covers it" => "被窗口遮挡时淡出",
        "Close the panel when the pointer leaves" => "指针移开后收起面板",
        "Smart hints (battery, break, copied link)" => "智能提示（电量、休息、复制的链接）",
        "Chime for meeting, battery and break hints" => "会议、电量和休息提示时播放提示音",
        "Auto break after focus (5 min)" => "专注后自动休息（5 分钟）",
        "CPU and memory on the bar" => "顶栏显示 CPU 和内存",
        "Reserve screen space (when top bar is off)" => "预留屏幕空间（关闭顶栏时）",
        "Buddy" => "桌面伙伴",
        "Robot (default)" => "机器人（默认）",
        "Kitty" => "猫咪",
        "Ronin (pixel, talks)" => "浪人（像素风，会说话）",
        "Courier (pixel, talks)" => "信使（像素风，会说话）",
        "Necromancer (pixel, talks)" => "死灵法师（像素风，会说话）",
        "Idle glance" => "闲置速览",
        "Tasks left" => "待办剩余",
        "Focus today" => "今日专注",
        "CPU and RAM" => "CPU 和内存",
        "Next meeting" => "下个会议",
        "Show focus progress on the notch" => "刘海显示专注进度",
        "Workday progress line on the notch" => "刘海显示工作日进度线",
        "Scroll up over the notch cycles time, date, tasks" => "在刘海上方滚轮切换时间、日期、待办",
        "State dot on the notch (mic, battery, focus)" => "刘海状态点（麦克风、电量、专注）",
        "Buddy peeks over the notch now and then" => "伙伴不时从刘海探出头",
        "Scenes" => "小剧场",
        "Play scenes now and then" => "偶尔自动播放小剧场",
        "Play: Showdown" => "播放：决战",
        "Play: Heist" => "播放：神偷行动",
        "Play: Team-up" => "播放：组队出击",
        "Play: Duel at dusk" => "播放：黄昏决斗",
        "Play: Rise" => "播放：觉醒",
        "Play: Alley fight" => "播放：巷战",
        "Play: Harry vs Voldemort" => "播放：哈利大战伏地魔",
        "Play: Owl post" => "播放：猫头鹰来信",
        "Play: Harry and friends vs Voldemort" => "播放：哈利与伙伴大战伏地魔",
        "Play: Quidditch" => "播放：魁地奇",
        "Play: Dementor" => "播放：摄魂怪",
        "Play: Friday deploy" => "播放：周五上线",
        "Play: You're on mute" => "播放：你静音了",
        "Play: Low battery" => "播放：电量不足",
        "Play: Cricket six" => "播放：板球六分",
        "Clock style" => "时钟样式",
        "Digital" => "数字",
        "Analog" => "模拟",
        "Binary" => "二进制",
        "Hours until Friday" => "距离周五小时数",
        "Water reminder" => "喝水提醒",
        "Look-away reminder (eyes)" => "远眺提醒（护眼）",
        "Every 20 minutes" => "每 20 分钟",
        "Every 30 minutes" => "每 30 分钟",
        "Every 45 minutes" => "每 45 分钟",
        "Every hour" => "每小时",
        "Every 90 minutes" => "每 90 分钟",
        "Primary display" => "主显示器",
        "Show on display" => "显示位置",
        "Display {i}" => "显示器 {i}",
        "Start with Windows" => "开机自启",
        "Show in Start menu and search" => "在开始菜单和搜索中显示",
        "Edit pinned apps" => "编辑固定应用",
        "Reload pinned apps" => "重新加载固定应用",
        "Open settings folder" => "打开设置文件夹",
        "Check for updates" => "检查更新",
        "Quit" => "退出",

        // ------------------------------------------------------- dialogs etc
        "Couldn't check for updates. Are you online?" => "无法检查更新。请确认网络连接。",
        "Nock {cur} is the latest version." => "Nock {cur} 已是最新版本。",
        "Nock {v} is available (you have {cur}).\n\nUpdate now? Nock restarts by itself in a few seconds." => {
            "发现新版本 Nock {v}（当前 {cur}）。\n\n要现在更新吗？Nock 将在几秒后自动重启。"
        }
        "Nock {v} is available (you have {cur}).\n\nYes: update now (Nock downloads it and restarts).\nNo: open the download page instead." => {
            "发现新版本 Nock {v}（当前 {cur}）。\n\n是：立即更新（下载后自动重启）。\n否：打开下载页面。"
        }
        "Nock - a notch and top bar for Windows" => "Nock - Windows 刘海与顶栏",
        "To-do list is full" => "待办列表已满",
        "Battery is low, find a charger" => "电量偏低，记得找充电器",
        "You have been at it for 90 minutes. Stretch!" => "已连续用机 90 分钟，起来伸个懒腰！",
        "Look at something far away for 20 seconds" => "看看远处，放松 20 秒",
        "Time to drink some water!" => "该喝口水了！",
        "Link copied" => "链接已复制",
        "Code copied" => "代码已复制",
        "Colour copied" => "颜色已复制",
        "Path copied" => "路径已复制",
        "in {mins} min" => "{mins} 分钟后",
        "{} min" => "{} 分钟",
        "{} left" => "剩 {} 项",
        "CPU {cpu}%  RAM {mem}%" => "CPU {cpu}%  内存 {mem}%",
        "CPU {c}% RAM {m}%" => "CPU {c}% 内存 {m}%",

        _ => s,
    }
}

/// `tr` plus `{name}` placeholder substitution, for sentences built at runtime
/// (`format!` needs a literal template, so templates live in the table).
pub fn tf(s: &str, subs: &[(&str, &str)]) -> String {
    let mut out = tr(s).to_string();
    for (k, v) in subs {
        out = out.replace(k, v);
    }
    out
}
