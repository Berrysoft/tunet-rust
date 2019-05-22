use tunet_rust::strfmt;
use tunet_rust::*;

fn main() -> Result<()> {
    let c = from_state(NetState::Auth4).unwrap();
    let f = c.flux()?;
    println!("用户：{}", f.username);
    println!("流量：{}", strfmt::format_flux(f.flux));
    println!("时长：{}", strfmt::format_duration(f.online_time));
    println!("余额：¥{:.2}", f.balance);
    Ok(())
}
