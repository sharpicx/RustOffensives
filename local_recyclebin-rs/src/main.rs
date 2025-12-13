use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use windows::Win32::System::Variant::*;
use windows::Win32::{System::Com::*, UI::Shell::*};
use windows_core::*;

pub struct RecycleBinDTO {
    pub name: String,
    pub path: String,
    pub size: i32,
    pub deleted_from: String,
    pub date_deleted: NaiveDateTime,
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

fn main() -> windows_core::Result<()> {
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
                    let prop_name = BSTR::from("System.Recycle.DeletedFrom");
                    let prop_type: VARIANT = item.ExtendedProperty(&prop_name)?;
                    if prop_type.vt() == VT_BSTR {
                        let deleted_from =
                            unsafe { prop_type.Anonymous.Anonymous.Anonymous.bstrVal.to_string() };
                        let dto = RecycleBinDTO {
                            name: name.to_string(),
                            path: path.to_string(),
                            size: size,
                            deleted_from: deleted_from,
                            date_deleted: datetime,
                        };
                        println!(
                            "Name: {}\nPath: {}\nSize: {}\nDeletedFrom: {}\nDateDeleted: {}",
                            dto.name,
                            dto.path,
                            dto.size,
                            dto.deleted_from,
                            dto.date_deleted.format("%d-%m-%Y %H:%M:%S")
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
