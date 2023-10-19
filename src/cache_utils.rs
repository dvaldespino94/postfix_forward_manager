use egui::Ui;

// Helper to get cached values from the interface
pub fn get_cache_value<T>(id: &str, ui: &mut Ui) -> T
where
    T: Default + Clone + Send + Sync + 'static,
{
    ui.memory_mut(|writer| {
        writer
            .data
            .get_temp_mut_or_insert_with((id.to_owned() + "v").into(), || -> T {
                Default::default()
            })
            .to_owned()
    })
}

// Helper to set cached values
pub fn set_cache_value<T>(id: &str, ui: &mut Ui, value: T)
where
    T: Default + Clone + Send + Sync + 'static,
{
    ui.memory_mut(|writer| {
        let result: &mut T = writer
            .data
            .get_temp_mut_or_default((id.to_owned() + "v").into());
        *result = value;
    })
}
