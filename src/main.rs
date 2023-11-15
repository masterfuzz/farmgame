use comfy::*;

comfy_game!("Nice red circle", FarmGame);

#[derive(Clone)]
struct Crop {
    name: String,
    stage: u32,
    max_stage: u32,
    value: u32,
}

struct Seed {
    name: String,
    yields: String,
    value: u32,
}

#[derive(Clone)]
struct Plot {
    crop: Option<Crop>,
}

type Inventory = Vec<(String, u32)>;

enum GameItem {
    Seed(Seed),
    Crop(Crop),
}

struct FarmGame {

    selected_circle: Option<Vec2>,

    selected_seed: Option<String>,

    plots: Vec<Plot>,

    inventory: Inventory,

    money: u32,

    events: Vec<GameEvent>,

    available_items: HashMap<String, GameItem>,
}

enum GameEvent {
    AddItemToInventory(String, u32),
    RemoveItemFromInventory(String, u32),
    PlantFromInventory(String, usize, usize),
}


impl GameLoop for FarmGame {
    fn new(_c: &mut comfy::EngineState) -> Self {
        let mut available_items: HashMap<String, GameItem> = HashMap::new();
        available_items.insert("carrot seeds".to_string(), GameItem::Seed(Seed{name: "carrot seeds".to_string(), value: 10, yields: "carrots".to_string()}));
        available_items.insert("pumpkin seeds".to_string(), GameItem::Seed(Seed{name: "pumpkin seeds".to_string(), value: 20, yields: "pumpkins".to_string()}));
        available_items.insert("carrots".to_string(), GameItem::Crop(Crop{name: "carrots".to_string(), stage: 0, max_stage: 5, value: 50}));
        available_items.insert("pumpkins".to_string(), GameItem::Crop(Crop{name: "pumpkins".to_string(), stage: 0, max_stage: 10, value: 100}));

        Self {
            selected_circle: None,
            selected_seed: Some("carrot seeds".to_string()),
            plots: vec![Plot { crop: None }; 16*16],
            inventory: vec![("carrot seeds".to_string(), 10)],
            money: 0,
            events: vec![],

            available_items,
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
                GameEvent::PlantFromInventory(item, x, y) => {
                    if let Some(GameItem::Seed(seed)) = self.available_items.get(&item) {

                        if !remove_item_from_inventory(&mut self.inventory, item.clone(), 1) {
                            println!("Tried to remove {} {} from inventory but there was none", 1, item);
                        } else {
                            self.plots[x + y * 16].crop = Some(get_crop(&self.available_items, &seed.yields).clone());
                        }
                    } else {
                        println!("Tried to plant {} but it was not a seed", item);
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
                let plot = &mut self.plots[(x+8) as usize + (y+8) as usize * 16];

                if let Some(crop) = &mut plot.crop {
                    if crop.stage == crop.max_stage {
                        self.events.push(GameEvent::AddItemToInventory(crop.name.clone(), 1));
                        plot.crop = None;
                    }
                } else {
                    if let Some(seed_name) = &self.selected_seed {
                        let seed = get_seed(&self.available_items, seed_name);
                        let plot = &mut self.plots[(x+8) as usize + (y+8) as usize * 16];
                        if remove_item_from_inventory(&mut self.inventory, seed.name.clone(), 1) {
                            plot.crop = Some(get_crop(&self.available_items, &seed.yields).clone());
                        } else {
                            println!("Tried to plant {} but it was not in inventory", seed.name);
                            self.selected_seed = None;
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
                    if let Some(item) = self.available_items.get(&item.0) {
                        match item {
                            GameItem::Seed(seed) => {
                                self.selected_seed = Some(seed.name.clone());
                            }
                            GameItem::Crop(crop) => {
                                self.events.push(GameEvent::RemoveItemFromInventory(crop.name.clone(), 1));
                                self.money += crop.value;
                            }
                        }
                    }
                }
            }
            ui.label(&format!("Money: {}", self.money));
            ui.label(&format!("Selected seed: {}", if let Some(seed) = &self.selected_seed { seed.clone() } else { "None".to_string() }));
        });

        egui::Window::new("Shop").show(egui(), |ui| {
            ui.label("Shop:");
            for item in self.available_items.values() {
                if let GameItem::Seed(seed) = item {
                    if ui.button(&format!("{}: {}$", seed.name, seed.value)).clicked() {
                        if self.money >= seed.value {
                            self.money -= seed.value;
                            self.events.push(GameEvent::AddItemToInventory(seed.name.clone(), 1));
                        }
                    }
                }
            }
        });

        
        
    }



}

fn get_seed<'a>(available_items: &'a HashMap<String, GameItem>, name: &str) -> &'a Seed {
    if let Some(GameItem::Seed(seed)) = available_items.get(name) {
        seed
    } else {
        panic!("Tried to get seed {} but it was not a seed", name);
    }
}

fn get_crop<'a>(available_items: &'a HashMap<String, GameItem>, name: &str) -> &'a Crop {
    if let GameItem::Crop(crop) = &available_items[name] {
        crop
    } else {
        panic!("Tried to get crop {} but it was not a crop", name);
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

