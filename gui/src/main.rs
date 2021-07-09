use std::{collections::HashMap, env};

use common::{application_bindings::ApplicationBindings, serial::{self, CONFIG_DIR}, xrapplication_info::{XrApplicationInfo}};
use iced::{Application, Button, Column, Command, Element, Row, Scrollable, Settings, Text, TextInput, button, executor, scrollable, text_input};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    BindingsGUI::run(Settings::with_flags(args.get(1).unwrap().clone())).unwrap();
}

pub struct BindingsGUI {
    application_info: XrApplicationInfo,
    default_bindings: ApplicationBindings,

    refresh_button_state: button::State,
    scroll_state: scrollable::State,

    input_states: HashMap<String, HashMap<String, HashMap<String, ActionWidgetState>>>,
    input_values: Vec<String>,
}

struct ActionWidgetState {
    localized_name: String,
    sub_actions: Vec<BindingWidgetState>,
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
                scroll_state: scrollable::State::new(),
                input_states: HashMap::new(),
                input_values: Vec::new(),
                application_info: XrApplicationInfo {
                    action_sets: HashMap::new(),
                    application_name: flags
                },
                default_bindings: ApplicationBindings {
                    profiles: HashMap::new()
                }
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from(format!("OxideXR for {}", self.application_info.application_name))
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match message {
            Message::Refresh => {
                let uuid = serial::get_uuid(&self.application_info.application_name);
                let file_path = format!("{}{}/actions.json", CONFIG_DIR, uuid);
                self.application_info = serial::read_json::<XrApplicationInfo>(&file_path).unwrap();
                let file_path = format!("{}{}/default_bindings.json", CONFIG_DIR, uuid);
                self.default_bindings = serial::read_json::<ApplicationBindings>(&file_path).unwrap();

                self.input_states.clear();
                for (profile_name, profile_binding) in &self.default_bindings.profiles {
                    let mut action_sets = HashMap::new();

                    for (set_name, set_info) in &self.application_info.action_sets {
                        let mut actions = HashMap::new();
                        let action_set_bindings = profile_binding.action_sets.get(set_name);

                        for (action_name, action_info) in &set_info.actions {
                            let mut aws = ActionWidgetState {
                                localized_name: action_info.localized_name.clone(),
                                sub_actions: Vec::new(),
                            };

                            if action_info.subaction_paths.is_empty() {
                                let mut binding = None; 
                                if let Some(action_set_bindings) = action_set_bindings {
                                    if let Some(action_bindings) = action_set_bindings.actions.get(action_name) {
                                        binding = Some(action_bindings.bindings[0].clone());    
                                    }
                                }
                                self.input_values.push(binding.unwrap_or_default());
                                aws.sub_actions.push(BindingWidgetState {
                                    input_state: text_input::State::new(),
                                    sub_action_path: None,
                                    path_str_idx: self.input_values.len() - 1,
                                });
                            } else {
                                for subaction_path in &action_info.subaction_paths {
                                    let mut binding = None; 
                                    if let Some(action_set_bindings) = action_set_bindings {
                                        if let Some(action_bindings) = action_set_bindings.actions.get(action_name) {
                                            binding = action_bindings.bindings.iter().find(|binding| {binding.starts_with(subaction_path)}).cloned();
                                        }
                                    }
                                    self.input_values.push(binding.unwrap_or_default());
                                    aws.sub_actions.push(BindingWidgetState {
                                        input_state: text_input::State::new(),
                                        sub_action_path: Some(subaction_path.clone()),
                                        path_str_idx: self.input_values.len() - 1,
                                    });
                                }
                            }

                            actions.insert(action_name.clone(), aws);
                        }
                        action_sets.insert(set_name.clone(), actions);
                    }
                    self.input_states.insert(profile_name.clone(), action_sets);
                }
            },
            Message::None => (),
            Message::UpdateText(string, idx) => {
                self.input_values[idx] = string;
            },
        }
        
        Command::none()
    }

    fn view<'a>(&mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new()
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh));

            //TODO clean this up with rustic iteration
            for (profile_name, action_sets) in &mut self.input_states {
                column = column.push(Text::new(profile_name).size(35));
                for (set_name, actions) in action_sets {
                    column = column.push(Text::new(set_name).size(30));
                    for (_, aws) in actions {
                        for bs in &mut aws.sub_actions {
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