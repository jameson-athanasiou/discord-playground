use chrono;

pub fn get_title_date(backset: u64) -> String {
    let today = chrono::offset::Local::now()
        .checked_sub_days(chrono::Days::new(backset))
        .unwrap();

    let formatted_today = today.format("%m-%d-%Y").to_string();

    formatted_today
}
