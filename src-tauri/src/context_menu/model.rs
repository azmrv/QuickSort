use crate::models::Folder;

pub struct MenuModel {
    pub items: Vec<MenuItem>,
}

pub enum MenuItem {
    Favorite { id: String, name: String, target: String },
    More,
}

impl MenuModel {
    pub fn from_folders(folders: &[Folder]) -> Self {
        let mut items: Vec<MenuItem> = folders
            .iter()
            .filter(|f| f.favorite)
            .enumerate()
            .map(|(i, f)| MenuItem::Favorite {
                id: format!("{:03}_{}", i, f.name),
                name: f.name.clone(),
                target: f.path.to_string_lossy().into(),
            })
            .collect();
        items.push(MenuItem::More); 
        Self { items }
    }
}
