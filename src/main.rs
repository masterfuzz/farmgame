use comfy::*;

comfy_game!("Nice red circle", FarmGame);

#[derive(Clone)]
struct Crop {
    name: String,
    stage: u32,
    max_stage: u32,
}

struct Seed {
    name: String,
    yields: Crop,
}

#[derive(Clone)]
struct Plot {
    crop: Option<Crop>,
}

type Inventory = Vec<(String, u32)>;


struct FarmGame {

    selected_circle: Option<Vec2>,

    selected_seed: Option<Seed>,

    plots: Vec<Plot>,

    inventory: Inventory,

    money: u32,

    events: Vec<GameEvent>,

}

enum GameEvent {
    AddItemToInventory(String, u32),
    RemoveItemFromInventory(String, u32),
}


impl GameLoop for FarmGame {
    fn new(_c: &mut comfy::EngineState) -> Self {
        Self {
            selected_circle: None,
            selected_seed: Some(Seed{name: "carrot seeds".to_string(), yields: Crop{name: "carrots".to_string(), stage: 0, max_stage: 10}}),
            plots: vec![Plot { crop: None }; 16*16],
            inventory: vec![("carrot seeds".to_string(), 10)],
            money: 0,
            events: vec![],
        }
    }

    fn update(&mut self, _c: &mut EngineContext) {
        let tick = if get_frame() % 60 == 0 { 1 } else { 0 };
        // draw a 16x16 grid of circles
        for x in -8..8 {
            for y in -8..8 {
                if let Some(crop) = &mut self.plots[(x+8) as usize + (y+8) as usize * 16].crop {
                    if crop.stage < crop.max_stage {
                        // tick every 60 frames
                        crop.stage += tick;
                        // crop.stage += 1;
                        draw_text(
                            &format!("{}", crop.stage),
                            vec2(x as f32, y as f32),
                            WHITE,
                            TextAlign::Center,
                        );
                        draw_circle(vec2(x as f32, y as f32), 0.5, GREEN, 0);
                    } else {
                        draw_circle(vec2(x as f32, y as f32), 0.5, YELLOW, 0);
                    }
                } else {
                                draw_circle(vec2(x as f32, y as f32), 0.5, WHITE, 0);
                                                }
                // draw_circle(vec2(x as f32, y as f32), 0.5, WHITE, 0);
            }
        }

        for event in self.events.drain(..) {
            match event {
                GameEvent::AddItemToInventory(item, amount) => {
                    add_item_to_inventory(&mut self.inventory, item.clone(), amount);
                },
                GameEvent::RemoveItemFromInventory(item, amount) => {
                    if !remove_item_from_inventory(&mut self.inventory, item.clone(), amount) {
                        println!("Tried to remove {} {} from inventory but there was none", amount, item);
                    }
                },
            }
        }


        if is_mouse_button_down(MouseButton::Left) {
            let m = mouse_world() ;
            let x = m.x.round() as i32;
            let y = m.y.round() as i32;
            if x >= -8 && x < 8 && y >= -8 && y < 8 {
                let c = vec2(x as f32, y as f32);

                self.selected_circle = Some(c);
                if let Some(seed) = &self.selected_seed {
                    let plot = &mut self.plots[(x+8) as usize + (y+8) as usize * 16];
                    if let Some(crop) = &mut plot.crop {
                        if crop.stage == crop.max_stage {
                            // add_item_to_inventory(&mut self.inventory, crop.name.clone(), 1);
                            self.events.push(GameEvent::AddItemToInventory(crop.name.clone(), 1));
                            plot.crop = None;
                        }
                    } else {
                        if remove_item_from_inventory(&mut self.inventory, seed.name.clone(), 1) {
                            plot.crop = Some(seed.yields.clone());
                        }
                    }
                }
            } else {
                self.selected_circle = None;
            }

        }

        if let Some(c) = self.selected_circle {
            draw_circle_outline(c, 0.5, 0.2, RED, 10);
        }

        egui::Window::new("Inventory").show(egui(), |ui| {
            ui.label("Inventory:");
            for item in self.inventory.iter() {
                if ui.button(&format!("{}: {}", item.0, item.1)).clicked() {
                    if item.0 == "carrot seeds" {
                        self.selected_seed = Some(Seed{name: "carrot seeds".to_string(), yields: Crop{name: "carrots".to_string(), stage: 0, max_stage: 10}});
                    }
                    if item.0 == "carrots" {
                        // add_item_to_inventory(&mut self.inventory, "money".to_string(), 5);
                        self.events.push(GameEvent::RemoveItemFromInventory("carrots".to_string(), 1));
                        
                        self.money += 50;
                    }
                }
                // draw_text(&format!("{}: {}", item.0, item.1), vec2(0.0, 0.0), WHITE, TextAlign::TopLeft);
            }
            ui.label(&format!("Money: {}", self.money));
            ui.label(&format!("Selected seed: {}", if let Some(seed) = &self.selected_seed { seed.name.clone() } else { "None".to_string() }));
        });

        egui::Window::new("Shop").show(egui(), |ui| {
            ui.label("Shop:");
            if ui.button("Buy carrot seeds").clicked() {
                if self.money >= 10 {
                    self.money -= 10;
                    self.events.push(GameEvent::AddItemToInventory("carrot seeds".to_string(), 1));
                }
            }
        });

        
        
    }


}

fn add_item_to_inventory(inventory: &mut Inventory, item: String, amount: u32) {
    for i in 0..inventory.len() {
        if inventory[i].0 == item {
            inventory[i].1 += amount;
            return;
        }
    }
    inventory.push((item, amount));
}

fn remove_item_from_inventory(inventory: &mut Inventory, item: String, amount: u32) -> bool {
    for i in 0..inventory.len() {
        if inventory[i].0 == item {
            inventory[i].1 -= amount;
            if inventory[i].1 == 0 {
                inventory.remove(i);
            }
            return true;
        }
    }
    return false;
}

