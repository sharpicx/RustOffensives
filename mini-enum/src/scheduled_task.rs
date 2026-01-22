use chrono::TimeZone;
use chrono::{DateTime, Local, NaiveDateTime};
use std::collections::HashMap;
use std::ptr::null_mut;
use winapi::shared::rpcdce::{RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE};
use winapi::shared::winerror::S_OK;
use winapi::shared::wtypes::{
    BSTR, VT_ARRAY, VT_BOOL, VT_BSTR, VT_DATE, VT_EMPTY, VT_I2, VT_I4, VT_NULL, VT_UNKNOWN,
};
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::um::combaseapi::*;
use winapi::um::oaidl::{SAFEARRAY, VARIANT};
use winapi::um::objbase::*;
use winapi::um::objidl::EOAC_NONE;
use winapi::um::oleauto::*;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::wbemcli::*;
use winapi::um::winnt::HRESULT;

// winapi-0.3.9\src\um\oleauto.rs
#[link(name = "oleaut32")]
unsafe extern "system" {
    fn SafeArrayGetElement(
        psa: *mut SAFEARRAY,
        rgIndices: *const i32,
        pv: *mut std::ffi::c_void,
    ) -> HRESULT;
}

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
enum RunLevelEnum {
    Lua = 0,
    Highest = 1,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
enum StateEnum {
    Unknown = 0,
    Disabled = 1,
    Queued = 2,
    Ready = 3,
    Running = 4,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum SecurityLogonType {
    Interactive = 2,
    Network = 3,
    Batch = 4,
    Service = 5,
    Proxy = 6,
    Unlock = 7,
    NetworkCleartext = 8,
    NewCredentials = 9,
    RemoteInteractive = 10,
    CachedInteractive = 11,
    CachedRemoteInteractive = 12,
    CachedUnlock = 13,
}

impl TryFrom<i32> for RunLevelEnum {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(RunLevelEnum::Lua),
            1 => Ok(RunLevelEnum::Highest),
            _ => Err(()),
        }
    }
}

impl TryFrom<i32> for StateEnum {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(StateEnum::Unknown),
            1 => Ok(StateEnum::Disabled),
            2 => Ok(StateEnum::Queued),
            3 => Ok(StateEnum::Ready),
            4 => Ok(StateEnum::Running),
            _ => Err(()),
        }
    }
}

impl TryFrom<i32> for SecurityLogonType {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            2 => Ok(SecurityLogonType::Interactive),
            3 => Ok(SecurityLogonType::Network),
            4 => Ok(SecurityLogonType::Batch),
            5 => Ok(SecurityLogonType::Service),
            6 => Ok(SecurityLogonType::Proxy),
            7 => Ok(SecurityLogonType::Unlock),
            8 => Ok(SecurityLogonType::NetworkCleartext),
            9 => Ok(SecurityLogonType::NewCredentials),
            10 => Ok(SecurityLogonType::RemoteInteractive),
            11 => Ok(SecurityLogonType::CachedInteractive),
            12 => Ok(SecurityLogonType::CachedRemoteInteractive),
            13 => Ok(SecurityLogonType::CachedUnlock),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for RunLevelEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RunLevelEnum::Lua => "TASK_RUNLEVEL_LUA",
            RunLevelEnum::Highest => "TASK_RUNLEVEL_HIGHEST",
        };
        f.write_str(s)
    }
}

impl std::fmt::Display for StateEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StateEnum::Unknown => "Unknown",
            StateEnum::Disabled => "Disabled",
            StateEnum::Queued => "Queued",
            StateEnum::Ready => "Ready",
            StateEnum::Running => "Running",
        };
        f.write_str(s)
    }
}

impl std::fmt::Display for SecurityLogonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SecurityLogonType::Interactive => "Interactive",
            SecurityLogonType::Network => "Network",
            SecurityLogonType::Batch => "Batch",
            SecurityLogonType::Service => "Service",
            SecurityLogonType::Proxy => "Proxy",
            SecurityLogonType::Unlock => "Unlock",
            SecurityLogonType::NetworkCleartext => "NetworkCleartext",
            SecurityLogonType::NewCredentials => "NewCredentials",
            SecurityLogonType::RemoteInteractive => "RemoteInteractive",
            SecurityLogonType::CachedInteractive => "CachedInteractive",
            SecurityLogonType::CachedRemoteInteractive => "CachedRemoteInteractive",
            SecurityLogonType::CachedUnlock => "CachedUnlock",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
struct ScheduledTaskPrincipal {
    display_name: Option<String>,
    group_id: Option<String>,
    id: Option<String>,
    user_id: Option<String>,
    logon_type: Option<SecurityLogonType>,
    run_level: Option<RunLevelEnum>,
}

#[derive(Debug)]
struct ScheduledTaskAction {
    properties: HashMap<String, String>,
}

#[derive(Debug)]
struct ScheduledTaskTrigger {
    enabled: Option<bool>,
    duration: Option<String>,
    interval: Option<String>,
    stop_at_duration_end: Option<bool>,
    properties: HashMap<String, String>,
}

#[derive(Debug)]
struct ScheduledTaskSettings {
    enabled: Option<bool>,
    hidden: Option<bool>,
    allow_demand_start: Option<bool>,
    allow_hard_terminate: Option<bool>,
    disallow_start_if_on_batteries: Option<bool>,
    stop_if_going_on_batteries: Option<bool>,
    execution_time_limit: Option<String>,
}

#[derive(Debug)]
struct ScheduledTask {
    task_name: Option<String>,
    state: Option<StateEnum>,
    author: Option<String>,
    source: Option<String>,
    task_path: Option<String>,
    date: Option<String>,
    description: Option<String>,
    security_descriptor: Option<String>,
    principal: ScheduledTaskPrincipal,
    actions: Vec<ScheduledTaskAction>,
    triggers: Vec<ScheduledTaskTrigger>,
    settings: Option<ScheduledTaskSettings>,
}

fn wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

fn variant_to_string(value: &VARIANT) -> Option<String> {
    unsafe {
        let vt = value.n1.n2().vt as u32;
        match vt {
            VT_BSTR => {
                let bstr = *value.n1.n2().n3.bstrVal();
                if bstr.is_null() {
                    return None;
                }
                let len = SysStringLen(bstr);
                let slice = std::slice::from_raw_parts(bstr, len as usize);
                let s = String::from_utf16_lossy(slice);
                if !s.trim().is_empty() {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                        let formatted = dt.format("%A, %d %B %Y %I:%M %p").to_string();
                        Some(formatted)
                    } else if let Ok(ndt) =
                        NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
                    {
                        let dt: DateTime<Local> = Local.from_local_datetime(&ndt).unwrap();
                        Some(dt.format("%A, %d %B %Y %I:%M %p").to_string())
                    } else {
                        Some(s)
                    }
                } else {
                    Some(String::new())
                }
            }
            VT_DATE => {
                let date_val = *value.n1.n2().n3.date();
                let base = chrono::NaiveDate::from_ymd(1899, 12, 30).and_hms(0, 0, 0);
                let whole_days = date_val.trunc() as i64;
                let fractional_day = date_val.fract();
                let seconds = (fractional_day * 24.0 * 3600.0).round() as i64;
                let dt =
                    base + chrono::Duration::days(whole_days) + chrono::Duration::seconds(seconds);
                Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            VT_I2 => {
                let i = *value.n1.n2().n3.iVal();
                Some(i.to_string())
            }
            VT_I4 => {
                let i = *value.n1.n2().n3.lVal();
                Some(i.to_string())
            }
            VT_EMPTY | VT_NULL => None,
            _ => Some(String::new()),
        }
    }
}

fn get_string_prop(obj: *mut IWbemClassObject, name: &str) -> Option<String> {
    unsafe {
        let mut value: VARIANT = std::mem::zeroed();
        let prop: BSTR = SysAllocString(wide(name).as_ptr());
        if prop.is_null() {
            return None;
        }
        let hr = (*obj).Get(
            prop,
            0,
            &mut value,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        SysFreeString(prop);
        if hr < 0 {
            VariantClear(&mut value);
            return None;
        }
        let ret = value.n1.n2().n3.bstrVal();
        if ret.is_null() {
            return None;
        }
        let result = variant_to_string(&value);
        VariantClear(&mut value);
        result
    }
}

fn get_embedded_object(obj: *mut IWbemClassObject, name: &str) -> Option<*mut IWbemClassObject> {
    unsafe {
        let mut val: VARIANT = std::mem::zeroed();
        let prop = SysAllocString(wide(name).as_ptr());
        (*obj).Get(prop, 0, &mut val, null_mut(), null_mut());
        SysFreeString(prop);
        if val.n1.n2().vt as u32 != VT_UNKNOWN {
            VariantClear(&mut val);
            return None;
        }
        let unk = *val.n1.n2().n3.punkVal();
        let mut embedded: *mut IWbemClassObject = null_mut();
        let hr = (*unk).QueryInterface(&IID_IWbemClassObject, &mut embedded as *mut _ as *mut _);
        VariantClear(&mut val);
        if hr < 0 { None } else { Some(embedded) }
    }
}

fn get_i32_prop(obj: *mut IWbemClassObject, name: &str) -> Option<i32> {
    unsafe {
        let mut val: VARIANT = std::mem::zeroed();
        let prop = SysAllocString(wide(name).as_ptr());
        (*obj).Get(prop, 0, &mut val, null_mut(), null_mut());
        SysFreeString(prop);
        let result = if val.n1.n2().vt as u32 == VT_BOOL || val.n1.n2().vt as u32 == VT_I4 {
            Some(*val.n1.n2().n3.lVal())
        } else {
            None
        };
        VariantClear(&mut val);
        result
    }
}

unsafe fn get_bool_prop(obj: *mut IWbemClassObject, name: &str) -> Option<bool> {
    let v = get_i32_prop(obj, name)?;
    Some(v != 0)
}

fn get_object_array(obj: *mut IWbemClassObject, name: &str) -> Vec<*mut IWbemClassObject> {
    unsafe {
        let mut val: VARIANT = std::mem::zeroed();
        let prop = SysAllocString(wide(name).as_ptr());

        (*obj).Get(prop, 0, &mut val, null_mut(), null_mut());
        SysFreeString(prop);

        let mut result = Vec::new();

        if val.n1.n2().vt as u32 == (VT_ARRAY | VT_UNKNOWN) {
            let psa = *val.n1.n2().n3.parray();
            let mut l = 0;
            let mut u = 0;

            SafeArrayGetLBound(psa, 1, &mut l);
            SafeArrayGetUBound(psa, 1, &mut u);

            for i in l..=u {
                let mut unk: *mut IUnknown = null_mut();
                SafeArrayGetElement(psa, &i, &mut unk as *mut _ as *mut _);

                let mut obj2: *mut IWbemClassObject = null_mut();
                (*unk).QueryInterface(&IID_IWbemClassObject, &mut obj2 as *mut _ as *mut _);
                result.push(obj2);
            }
        }

        VariantClear(&mut val);
        result
    }
}

// fn print_wbem_object(obj: *mut IWbemClassObject) {
//     unsafe {
//         if (*obj).BeginEnumeration(0) < 0 {
//             println!("Failed to begin enumeration");
//             return;
//         }
//
//         loop {
//             let mut name: BSTR = null_mut();
//             let mut val: VARIANT = std::mem::zeroed();
//             let hr = (*obj).Next(0, &mut name, &mut val, null_mut(), null_mut());
//             if hr < 0 || name.is_null() {
//                 break;
//             }
//             let prop_name = {
//                 let len = SysStringLen(name);
//                 let slice = std::slice::from_raw_parts(name, len as usize);
//                 String::from_utf16_lossy(slice)
//             };
//             SysFreeString(name);
//             let prop_value = match val.n1.n2().vt as u32 {
//                 VT_BSTR => {
//                     let bstr = *val.n1.n2().n3.bstrVal();
//                     if bstr.is_null() {
//                         "".to_string()
//                     } else {
//                         let len = SysStringLen(bstr);
//                         let slice = std::slice::from_raw_parts(bstr, len as usize);
//                         String::from_utf16_lossy(slice)
//                     }
//                 }
//                 VT_I4 => (*val.n1.n2().n3.lVal()).to_string(),
//                 VT_BOOL => ((*val.n1.n2().n3.boolVal()) != 0).to_string(),
//                 _ => format!("(vt=0x{:X})", val.n1.n2().vt as u32),
//             };
//
//             println!("{}: {}", prop_name, prop_value);
//
//             VariantClear(&mut val);
//         }
//
//         (*obj).EndEnumeration();
//     }
// }

fn build_principal(principal: *mut IWbemClassObject) -> ScheduledTaskPrincipal {
    let display_name = get_string_prop(principal, "DisplayName");
    let group_id = get_string_prop(principal, "GroupId");
    let id = get_string_prop(principal, "Id");
    let user_id = get_string_prop(principal, "UserId");
    let logon_type_i = get_i32_prop(principal, "LogonType");
    let run_level_i = get_i32_prop(principal, "RunLevel");
    let run_level = run_level_i.and_then(|v| RunLevelEnum::try_from(v).ok());
    let logon_type = logon_type_i.and_then(|v| SecurityLogonType::try_from(v).ok());
    ScheduledTaskPrincipal {
        display_name,
        group_id,
        id,
        user_id,
        logon_type,
        run_level,
    }
}

fn build_actions(action_objs: Vec<*mut IWbemClassObject>) -> Vec<ScheduledTaskAction> {
    unsafe {
        let mut actions = Vec::new();
        for obj in action_objs {
            if obj.is_null() {
                continue;
            }
            let mut props = HashMap::new();
            (*obj).BeginEnumeration(0);
            loop {
                let mut name: BSTR = null_mut();
                let mut val: VARIANT = std::mem::zeroed();
                let mut cimtype: i32 = 0;
                let mut flavor: i32 = 0;
                let hr = (*obj).Next(0, &mut name, &mut val, &mut cimtype, &mut flavor);
                if hr != S_OK || name.is_null() {
                    break;
                }
                let key = {
                    let len = SysStringLen(name);
                    let slice = std::slice::from_raw_parts(name, len as usize);
                    String::from_utf16_lossy(slice)
                };
                if val.n1.n2().vt as u32 == 0 || val.n1.n2().n3.bstrVal().is_null() {
                    continue;
                }
                SysFreeString(name);
                if key != "PSComputerName" {
                    let vt = val.n1.n2().vt as u32;
                    if vt == (VT_ARRAY | VT_BSTR) {
                        let psa = *val.n1.n2().n3.parray();
                        let mut lbound = 0;
                        let mut ubound = 0;
                        SafeArrayGetLBound(psa, 1, &mut lbound);
                        SafeArrayGetUBound(psa, 1, &mut ubound);
                        let mut values = Vec::new();
                        for i in lbound..=ubound {
                            let mut bstr_elem: BSTR = null_mut();
                            SafeArrayGetElement(psa, &i, &mut bstr_elem as *mut _ as *mut _);
                            if !bstr_elem.is_null() {
                                let len = SysStringLen(bstr_elem);
                                let slice = std::slice::from_raw_parts(bstr_elem, len as usize);
                                let s = String::from_utf16_lossy(slice);
                                values.push(s);
                            }
                        }
                        props.insert(key, format!("[{}]", values.join(", ")));
                    } else if let Some(v) = variant_to_string(&val) {
                        props.insert(key, v);
                    }
                }
                VariantClear(&mut val);
            }
            (*obj).EndEnumeration();
            actions.push(ScheduledTaskAction { properties: props });
            (*obj).Release();
        }
        actions
    }
}

fn build_triggers(task_objs: Vec<*mut IWbemClassObject>) -> Vec<ScheduledTaskTrigger> {
    unsafe {
        let mut triggers = Vec::new();
        for obj in task_objs {
            let enabled = get_bool_prop(obj, "Enabled");
            let mut duration = None;
            let mut interval = None;
            let mut stop_at_duration_end = None;
            if let rep = get_embedded_object(obj, "Repetition").unwrap() {
                duration = get_string_prop(rep, "Duration").filter(|s| !s.is_empty());
                interval = get_string_prop(rep, "Interval").filter(|s| !s.is_empty());
                stop_at_duration_end = get_bool_prop(rep, "StopAtDurationEnd");
                (*rep).Release();
            }
            let mut props = HashMap::new();
            (*obj).BeginEnumeration(0);
            loop {
                let mut name: BSTR = null_mut();
                let mut val: VARIANT = std::mem::zeroed();
                let mut cimtype: i32 = 0;
                let mut flavor: i32 = 0;
                let hr = (*obj).Next(0, &mut name, &mut val, &mut cimtype, &mut flavor);
                if hr != S_OK {
                    break;
                }
                let key = {
                    let len = SysStringLen(name);
                    let slice = std::slice::from_raw_parts(name, len as usize);
                    String::from_utf16_lossy(slice)
                };
                SysFreeString(name);
                let skip = matches!(
                    key.as_str(),
                    "Id" | "Enabled"
                        | "EndBoundary"
                        | "ExecutionTimeLimit"
                        | "StartBoundary"
                        | "Repetition"
                );
                if !skip {
                    if val.n1.n2().vt as u32 == (VT_ARRAY | VT_BSTR) {
                        let psa = *val.n1.n2().n3.parray();
                        let mut lbound = 0;
                        let mut ubound = 0;
                        SafeArrayGetLBound(psa, 1, &mut lbound);
                        SafeArrayGetUBound(psa, 1, &mut ubound);

                        let mut values = Vec::new();
                        for i in lbound..=ubound {
                            let mut bstr_elem: BSTR = null_mut();
                            SafeArrayGetElement(psa, &i, &mut bstr_elem as *mut _ as *mut _);
                            if !bstr_elem.is_null() {
                                let len = SysStringLen(bstr_elem);
                                let slice = std::slice::from_raw_parts(bstr_elem, len as usize);
                                let s = String::from_utf16_lossy(slice);
                                values.push(s);
                            }
                        }
                        props.insert(key, format!("[{}]", values.join(", ")));
                    } else if let Some(v) = variant_to_string(&val) {
                        props.insert(key, v);
                    }
                }
                VariantClear(&mut val);
            }
            (*obj).EndEnumeration();
            triggers.push(ScheduledTaskTrigger {
                enabled,
                duration,
                interval,
                stop_at_duration_end,
                properties: props,
            });
            (*obj).Release();
        }
        triggers
    }
}

fn build_settings(task_obj: *mut IWbemClassObject) -> Option<ScheduledTaskSettings> {
    unsafe {
        let settings = get_embedded_object(task_obj, "Settings")?;
        let s = ScheduledTaskSettings {
            enabled: get_bool_prop(settings, "Enabled"),
            hidden: get_bool_prop(settings, "Hidden"),
            allow_demand_start: get_bool_prop(settings, "AllowDemandStart"),
            allow_hard_terminate: get_bool_prop(settings, "AllowHardTerminate"),
            disallow_start_if_on_batteries: get_bool_prop(settings, "DisallowStartIfOnBatteries"),
            stop_if_going_on_batteries: get_bool_prop(settings, "StopIfGoingOnBatteries"),
            execution_time_limit: get_string_prop(settings, "ExecutionTimeLimit"),
        };
        (*settings).Release();
        Some(s)
    }
}

fn build_scheduled_task(obj: *mut IWbemClassObject) -> ScheduledTask {
    let task_name = get_string_prop(obj, "TaskName");
    let state = get_i32_prop(obj, "State").and_then(|v| StateEnum::try_from(v).ok());
    let author = get_string_prop(obj, "Author");
    let source = get_string_prop(obj, "Source");
    let task_path = get_string_prop(obj, "TaskPath");
    let date = get_string_prop(obj, "Date");
    // unsafe {
    //     let mut val: VARIANT = std::mem::zeroed();
    //     let prop = SysAllocString(wide("Date").as_ptr());
    //
    //     (*obj).Get(prop, 0, &mut val, null_mut(), null_mut());
    //     SysFreeString(prop);
    //
    //     println!("Raw VARIANT type for Date: 0x{:X}", val.n1.n2().vt as u32);
    //
    //     VariantClear(&mut val);
    // }
    let description = get_string_prop(obj, "Description");
    let security_descriptor = get_string_prop(obj, "SecurityDescriptor");

    let principal_obj = get_embedded_object(obj, "Principal").unwrap();
    let principal = build_principal(principal_obj);

    let actions_obj = get_object_array(obj, "Actions");
    let actions = build_actions(actions_obj);

    let triggers_obj = get_object_array(obj, "Triggers");
    let triggers = build_triggers(triggers_obj);

    let settings = build_settings(obj);

    ScheduledTask {
        task_name,
        state,
        author,
        source,
        task_path,
        date,
        description,
        security_descriptor,
        principal,
        actions,
        triggers,
        settings,
    }
}

fn print_scheduled_task(task: &ScheduledTask, indent: usize) {
    let pad = " ".repeat(indent);
    if let Some(name) = &task.task_name {
        if !name.is_empty() {
            println!("{pad}{}", name);
        }
    }
    if let Some(state) = task.state {
        println!("{pad}├── State: {}", state);
    }
    if let Some(author) = &task.author {
        if !author.is_empty() {
            println!("{pad}├── Author: {}", author);
        }
    }
    if let Some(source) = &task.source {
        if !source.is_empty() {
            println!("{pad}├── Source: {}", source);
        }
    }
    if let Some(task_path) = &task.task_path {
        if !task_path.is_empty() {
            println!("{pad}├── TaskPath: {}", task_path);
        }
    }
    if let Some(date) = &task.date {
        if !date.is_empty() {
            println!("{pad}├── Date: {}", date);
        }
    }
    if let Some(description) = &task.description {
        if !description.is_empty() {
            println!("{pad}├── Description: {}", description);
        }
    }
    if let Some(security_descriptor) = &task.security_descriptor {
        if !security_descriptor.is_empty() {
            println!("{pad}├── SecurityDescriptor: {}", security_descriptor);
        }
    }
    let p = &task.principal;
    if p.display_name.is_some() || p.id.is_some() || p.user_id.is_some() || p.group_id.is_some() {
        println!("{pad}├── Principal:");
        let ppad = format!("{pad}│   ");
        if let Some(v) = &p.display_name {
            if !v.is_empty() {
                println!("{ppad}├── DisplayName: {}", v);
            }
        }
        if let Some(v) = &p.id {
            if !v.is_empty() {
                println!("{ppad}├── Id: {}", v);
            }
        }
        if let Some(v) = &p.user_id {
            if !v.is_empty() {
                println!("{ppad}├── UserId: {}", v);
            }
        }
        if let Some(v) = &p.group_id {
            if !v.is_empty() {
                println!("{ppad}├── GroupId: {}", v);
            }
        }
        if let Some(v) = &p.logon_type {
            println!("{ppad}├── LogonType: {}", v);
        }

        if let Some(v) = p.run_level {
            println!("{ppad}└── RunLevel: {:?}", v);
        }
    }
    if !task.actions.is_empty() {
        println!("{pad}├── Actions:");
        let base = format!("{pad}│   ");
        let total = task.actions.len();
        for (i, a) in task.actions.iter().enumerate() {
            let is_last = i + 1 == total;
            let branch = if is_last { "└──" } else { "├──" };
            let next_pad = if is_last { "    " } else { "│   " };
            println!("{base}{branch} Action {}:", i + 1);
            let prop_pad = format!("{base}{next_pad}");
            for (k, v) in &a.properties {
                println!("{prop_pad}├── {k}: {v}");
            }
        }
    }
    if !task.triggers.is_empty() {
        println!("{pad}├── Triggers:");
        let base = format!("{pad}│   ");
        let total = task.triggers.len();
        for (i, t) in task.triggers.iter().enumerate() {
            let is_last = i + 1 == total;
            let branch = if is_last { "└──" } else { "├──" };
            let next_pad = if is_last { "    " } else { "│   " };
            println!("{base}{branch} Trigger {}:", i + 1);
            let field_pad = format!("{base}{next_pad}");
            println!("{field_pad}├── Enabled: {}", t.enabled.unwrap_or(false));
            if let Some(v) = &t.duration {
                println!("{field_pad}├── Duration: {}", v);
            }
            if let Some(v) = &t.interval {
                println!("{field_pad}├── Interval: {}", v);
            }
            if let Some(v) = t.stop_at_duration_end {
                println!("{field_pad}├── StopAtDurationEnd: {}", v);
            }
            if !t.properties.is_empty() {
                println!("{field_pad}└── Properties:");
                let prop_pad = format!("{field_pad}    ");
                for (k, v) in &t.properties {
                    println!("{prop_pad}├── {k}: {v}");
                }
            }
        }
    }
    if let Some(s) = &task.settings {
        println!("{pad}└── Settings:");
        let spad = format!("{pad}    ");

        if let Some(v) = s.enabled {
            println!("{spad}├── Enabled: {}", v);
        }
        if let Some(v) = s.hidden {
            println!("{spad}├── Hidden: {}", v);
        }
        if let Some(v) = s.allow_demand_start {
            println!("{spad}├── AllowDemandStart: {}", v);
        }
        if let Some(v) = s.allow_hard_terminate {
            println!("{spad}├── AllowHardTerminate: {}", v);
        }
        if let Some(v) = s.disallow_start_if_on_batteries {
            println!("{spad}├── DisallowStartIfOnBatteries: {}", v);
        }
        if let Some(v) = s.stop_if_going_on_batteries {
            println!("{spad}├── StopIfGoingOnBatteries: {}", v);
        }
        if let Some(v) = s.execution_time_limit.as_deref() {
            if !v.is_empty() {
                println!("{spad}└── ExecutionTimeLimit: {}", v);
            }
        }
    }
    println!();
}

pub fn enum_scheduled_tasks() {
    unsafe {
        CoInitializeEx(null_mut(), COINIT_MULTITHREADED);
        CoInitializeSecurity(
            null_mut(),
            -1,
            null_mut(),
            null_mut(),
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            null_mut(),
            EOAC_NONE,
            null_mut(),
        );
        let mut locator: *mut IWbemLocator = null_mut();
        let hr = CoCreateInstance(
            &CLSID_WbemLocator,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IWbemLocator,
            &mut locator as *mut _ as *mut _,
        );
        assert!(hr == S_OK);
        let namespace: BSTR =
            SysAllocString(wide("ROOT\\Microsoft\\Windows\\TaskScheduler").as_ptr());
        let mut services: *mut IWbemServices = null_mut();
        let null_bstr = BSTR::default();
        // https://docs.rs/winapi/latest/winapi/um/wbemcli/struct.IWbemLocator.html#method.ConnectServer
        (*locator).ConnectServer(
            namespace,     // strNetworkResource
            null_bstr,     // user
            null_bstr,     // password
            null_bstr,     // locale
            0,             // security flags
            null_bstr,     // authority
            null_mut(),    // IWbemContext*
            &mut services, // IWbemServices**
        );
        let mut enumerator: *mut IEnumWbemClassObject = null_mut();
        let query_language = SysAllocString(wide("WQL").as_ptr());
        let query = SysAllocString(wide("SELECT * FROM MSFT_ScheduledTask").as_ptr());
        let flags: i32 = (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32;
        (*services).ExecQuery(query_language, query, flags, null_mut(), &mut enumerator);
        loop {
            let mut obj: *mut IWbemClassObject = null_mut();
            let mut returned = 0;
            let hr = (*enumerator).Next(WBEM_INFINITE as i32, 1, &mut obj, &mut returned);
            if hr != S_OK || returned == 0 {
                break;
            }
            let task = build_scheduled_task(obj);
            print_scheduled_task(&task, 1);
            (*obj).Release();
        }

        assert!(hr == S_OK);
        SysFreeString(query_language);
        SysFreeString(query);
        SysFreeString(namespace);
        CoUninitialize();
    }
}
