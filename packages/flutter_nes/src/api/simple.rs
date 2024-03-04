use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn sum(a:usize,b:usize)->usize{
    a+b
}
pub fn sum_async(a:usize,b:usize)->usize{
    a+b
}