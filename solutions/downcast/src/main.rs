use downcast::liquid::Liquid;
use downcast::water::Water;
use downcast::oil::Oil;

fn main() {
    let liquids: Vec<Box<dyn Liquid>> = vec![
        Box::new(Water),
        Box::new(Oil),
        Box::new(Water),
    ];

    for liquid in &liquids {
        println!("Got a {}", liquid.description());

        // Downcast the trait object back to a concrete type
        if let Some(_water) = liquid.as_any().downcast_ref::<Water>() {
            println!("  -> successfully downcast to Water!");
        } else if let Some(_oil) = liquid.as_any().downcast_ref::<Oil>() {
            println!("  -> successfully downcast to Oil!");
        }
    }
}
