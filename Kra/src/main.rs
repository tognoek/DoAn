use redis::{Client, Commands, Connection, RedisResult};

fn main() -> RedisResult<()> {
    let client: Client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con: Connection = client.get_connection()?;
    let _: () = con.set("my_key", 42)?;
    let val: i32 = con.get("my_key")?;
    println!("mykey = {}", val);

    Ok(())
}
