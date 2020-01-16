pub fn up(n: usize) {
    print!("\x1B[{}A", n);
}

pub fn move_screen_up(n: usize) {
    print!("{}", "\n".repeat(n));
    up(n);
}
