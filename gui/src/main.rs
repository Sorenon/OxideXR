use std::{collections::HashMap, env, path::Path};

use common::{application_bindings::*, serial::{self, CONFIG_DIR}, xrapplication_info::XrApplicationInfo};
use iced::{Application, Button, Column, Command, Element, Row, Scrollable, Settings, Text, TextInput, button, executor, scrollable, text_input};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    BindingsGUI::run(Settings::with_flags(args.get(1).unwrap().clone())).unwrap();
}

pub struct BindingsGUI {
    application_name: String,

    refresh_button_state: button::State,
    save_button_state: button::State,
    scroll_state: scrollable::State,

    input_states: HashMap<String, HashMap<String, ActionSetWidgetState>>,
    input_values: Vec<String>,
}

struct ActionSetWidgetState {
    localized_name: String,
    actions: HashMap<String, ActionWidgetState>,
}

struct ActionWidgetState {
    localized_name: String,
    bindings: Vec<BindingWidgetState>,
}

struct BindingWidgetState {
    input_state: text_input::State,
    sub_action_path: Option<String>,
    path_str_idx: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    UpdateText(String, usize),
    Save,
    None,
}

impl Application for BindingsGUI {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = String;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            BindingsGUI {
                refresh_button_state: button::State::new(),
                save_button_state: button::State::new(),
                scroll_state: scrollable::State::new(),
                input_states: HashMap::new(),
                input_values: Vec::new(),
                application_name: flags,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from(format!("OxideXR for {}", self.application_name))
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match message {
            Message::Refresh => {
                let uuid = serial::get_uuid(&self.application_name);
                let file_path = format!("{}{}/actions.json", CONFIG_DIR, uuid);
                let application_info = serial::read_json::<XrApplicationInfo>(&file_path).unwrap();
                let file_path = format!("{}{}/default_bindings.json", CONFIG_DIR, uuid);
                let default_bindings = serial::read_json::<ApplicationBindings>(&file_path).unwrap();

                self.input_states.clear();
                for (profile_name, profile_binding) in &default_bindings.profiles {
                    let mut action_sets = HashMap::new();

                    for (set_name, set_info) in &application_info.action_sets {
                        let mut actions = HashMap::new();
                        let action_set_bindings = profile_binding.action_sets.get(set_name);

                        for (action_name, action_info) in &set_info.actions {
                            let mut aws = ActionWidgetState {
                                localized_name: action_info.localized_name.clone(),
                                bindings: Vec::new(),
                            };

                            if action_info.subaction_paths.is_empty() {
                                let mut binding = None; 
                                if let Some(action_set_bindings) = action_set_bindings {
                                    if let Some(action_bindings) = action_set_bindings.actions.get(action_name) {
                                        binding = Some(action_bindings.bindings[0].clone());    
                                    }
                                }
                                self.input_values.push(binding.unwrap_or_default());
                                aws.bindings.push(BindingWidgetState {
                                    input_state: text_input::State::new(),
                                    sub_action_path: None,
                                    path_str_idx: self.input_values.len() - 1,
                                });
                            } else {
                                for subaction_path in &action_info.subaction_paths {
                                    let mut binding = None; 
                                    if let Some(action_set_bindings) = action_set_bindings {
                                        if let Some(action_bindings) = action_set_bindings.actions.get(action_name) {
                                            let binding_opt = action_bindings.bindings.iter().find(|binding| {binding.starts_with(subaction_path)});
                                            if let Some(binding_opt) = binding_opt {
                                                binding = Some(binding_opt[subaction_path.len()..].to_owned());
                                            }
                                        }
                                    }
                                    self.input_values.push(binding.unwrap_or_default());
                                    aws.bindings.push(BindingWidgetState {
                                        input_state: text_input::State::new(),
                                        sub_action_path: Some(subaction_path.clone()),
                                        path_str_idx: self.input_values.len() - 1,
                                    });
                                }
                            }

                            actions.insert(action_name.clone(), aws);
                        }

                        action_sets.insert(set_name.clone(), ActionSetWidgetState {
                            localized_name: set_info.localized_name.clone(),
                            actions,
                        });
                    }
                    self.input_states.insert(profile_name.clone(), action_sets);
                }
            },
            Message::UpdateText(string, idx) => {
                self.input_values[idx] = string;
            },
            Message::Save => {
                let uuid = serial::get_uuid(&self.application_name);
                let file_path = format!("{}{}/bindings/custom_bindings.json", CONFIG_DIR, uuid);
                
                let mut custom_bindings = ApplicationBindings::default();
                for (profile_name, action_widget_sets) in &self.input_states {
                    let mut profile_bindings = InteractionProfileBindings::default();
                    for (set_name, action_widgets) in action_widget_sets {
                        let mut set_bindings = ActionSetBindings::default();
                        for (action_name, action_widget) in &action_widgets.actions {
                            let mut action_bindings = ActionBindings::default();
                            for binding in &action_widget.bindings {
                                let binding_path = &self.input_values[binding.path_str_idx];
                                if !binding_path.is_empty() {
                                    match &binding.sub_action_path {
                                        Some(sub_action_path) => action_bindings.bindings.push(format!("{}{}", sub_action_path, binding_path).to_owned()),
                                        None => action_bindings.bindings.push(binding_path.clone()),
                                    }
                                }
                            }
                            if !action_bindings.bindings.is_empty() {
                                set_bindings.actions.insert(action_name.clone(), action_bindings);
                            }
                        }
                        if !set_bindings.actions.is_empty() {
                            profile_bindings.action_sets.insert(set_name.clone(), set_bindings);
                        }
                    }
                    if !profile_bindings.action_sets.is_empty() {
                        custom_bindings.profiles.insert(profile_name.clone(), profile_bindings);
                    }
                }

                serial::write_json(&custom_bindings, Path::new(&file_path));
            },
            Message::None => (),
        }
        
        Command::none()
    }

    fn view<'a>(&mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new()
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh))
            .push(Button::new(&mut self.save_button_state, Text::new("Save")).on_press(Message::Save));

            for (profile_name, action_sets) in &mut self.input_states {
                column = column.push(Text::new(profile_name).size(35));
                for (_, action_set_widget) in action_sets {
                    column = column.push(Text::new(&action_set_widget.localized_name).size(30));
                    for (_, aws) in &mut action_set_widget.actions {
                        for bs in &mut aws.bindings {
                            let idx = bs.path_str_idx;
                            if let Some(subaction_path) = &bs.sub_action_path {
                                column = column.push(Row::new()
                                    .push(Text::new(&aws.localized_name).size(30))
                                    .push(Text::new("  ").size(30))
                                    .push(Text::new(subaction_path).size(30))
                                    .push(TextInput::new(&mut bs.input_state, "", self.input_values.get(idx).unwrap(), move |str| { Message::UpdateText(str, idx) }).size(30))
                                );
                            } else {
                                column = column.push(Row::new()
                                    .push(Text::new(&aws.localized_name).size(30))
                                    .push(Text::new("  ").size(30))
                                    .push(TextInput::new(&mut bs.input_state, "", self.input_values.get(idx).unwrap(), move |str| { Message::UpdateText(str, idx) }).size(30))
                                );
                            }
                        }    
                    }
                }
            }

        Scrollable::new(&mut self.scroll_state).push(column).into()
    }
}