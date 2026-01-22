use crate::helper;
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use colored::*;
use windows_alt::Win32::Foundation::VARIANT_BOOL;
use windows_alt::Win32::System::Variant::*;
use windows_alt::Win32::{System::Com::*, UI::Shell::*};
use windows_core::*;

struct RecycleBinDTO {
    name: String,
    type_str: String,
    is_folder: bool,
    is_link: bool,
    is_filesystem: bool,
    is_browsable: bool,
    path: String,
    size: i32,
    deleted_from: String,
    modify_date: NaiveDateTime,
    date_deleted: NaiveDateTime,
}

fn ole_date_to_naive(ole_date: f64) -> NaiveDateTime {
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let days = ole_date.trunc() as i64;
    let frac_day = ole_date.fract();
    let seconds = (frac_day * 24.0 * 3600.0).round() as i64;

    base + Duration::days(days) + Duration::seconds(seconds)
}

pub fn local_recyclebin() -> windows_core::Result<()> {
    unsafe {
        let last_days = 30;
        let start_time = Local::now() - Duration::days(last_days);
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let shell: IShellDispatch = CoCreateInstance(&Shell, None, CLSCTX_ALL)?;
        let ns_index = VARIANT::from(10i32);
        let recycle_bin = shell.NameSpace(&ns_index)?;
        let items = recycle_bin.Items()?;
        let count = items.Count()? as i32;
        for i in 0..count {
            let index = VARIANT::from(i);
            let item: FolderItem2 = items.Item(&index)?.cast()?;
            let prop_name = BSTR::from("System.Recycle.DateDeleted");
            let value: VARIANT = item.ExtendedProperty(&prop_name)?;
            if value.vt() == VT_DATE {
                let ole_date: f64 = value.Anonymous.Anonymous.Anonymous.date;
                let datetime = ole_date_to_naive(ole_date);
                if datetime > start_time.naive_local() {
                    let name = item.Name()?;
                    let path = item.Path()?;
                    let size = item.Size()?;
                    let is_browsable = item.IsBrowsable()? != VARIANT_BOOL(0);
                    let modify_date_raw: f64 = item.ModifyDate()?;
                    let modify_date: NaiveDateTime = ole_date_to_naive(modify_date_raw);
                    let type_str = item.Type()?;
                    let is_folder: bool = item.IsFolder()? != VARIANT_BOOL(0);
                    let is_link: bool = item.IsLink()? != VARIANT_BOOL(0);
                    let is_filesystem: bool = item.IsFileSystem()? != VARIANT_BOOL(0);
                    let prop_name = BSTR::from("System.Recycle.DeletedFrom");
                    let prop_type: VARIANT = item.ExtendedProperty(&prop_name)?;
                    if prop_type.vt() == VT_BSTR {
                        let deleted_from =
                            prop_type.Anonymous.Anonymous.Anonymous.bstrVal.to_string();
                        let dto = RecycleBinDTO {
                            name: name.to_string(),
                            type_str: type_str.to_string(),
                            is_folder,
                            is_link,
                            is_filesystem,
                            is_browsable,
                            path: path.to_string(),
                            size,
                            deleted_from,
                            modify_date,
                            date_deleted: datetime,
                        };
                        println!(
                            "[{}] Name: {}\n    Type: {}\n    IsFolder: {}\n    IsFileSystem: {}\n    IsLink: {}\n    IsBrowsable: {}\n    Path: {}\n    Size: {}\n    DeletedFrom: {}\n    ModifyDate: {}\n    DateDeleted: {}\n",
                            "+".bright_red(),
                            dto.name,
                            dto.type_str,
                            dto.is_folder,
                            dto.is_link,
                            dto.is_filesystem,
                            dto.is_browsable,
                            dto.path,
                            dto.size,
                            dto.deleted_from,
                            dto.modify_date.format("%d-%m-%Y %H:%M:%S"),
                            dto.date_deleted.format("%d-%m-%Y %H:%M:%S")
                        );
                    }
                }
            }
        }
    }

    helper::log_info("Check if the files are important to look at!\n");
    Ok(())
}
