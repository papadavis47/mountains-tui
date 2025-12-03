use crossterm::event::{KeyCode, KeyModifiers};

fn main() {
    let modifiers = KeyModifiers::SHIFT;
    println!("SHIFT contains SHIFT: {}", modifiers.contains(KeyModifiers::SHIFT));
    println!("SHIFT is SHIFT: {}", modifiers == KeyModifiers::SHIFT);
    
    let no_modifiers = KeyModifiers::empty();
    println!("empty contains SHIFT: {}", no_modifiers.contains(KeyModifiers::SHIFT));
}
