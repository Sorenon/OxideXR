use std::{collections::HashMap, env, ops::Add, path::Path};

use common::{application_bindings::*, interaction_profiles::Feature, serial::{self, CONFIG_DIR}, xrapplication_info::{ActionType, XrApplicationInfo}};
use iced::{Application, Button, Column, Command, Element, Row, Scrollable, Settings, Text, TextInput, button, executor, scrollable, text_input};

pub struct BindingsGUI {
    application_name: String,

    refresh_button_state: button::State,
    save_button_state: button::State,
    scroll_state: scrollable::State,

    input_states: HashMap<String, InteractionProfileWidget>,
}

#[derive(Debug)]
struct InteractionProfileWidget(HashMap<String, ActionSetWidget>);
#[derive(Debug)]
struct ActionSetWidget(HashMap<String, SubactionWidget>);
#[derive(Debug)]
struct SubactionWidget(HashMap<String, String>);

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
                // input_values: Vec::new(),
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

                let root = common::interaction_profiles::generate();

                let mut interaction_profile_widgets = HashMap::new();

                //TODO clean up this mess
                for (profile_name, profile) in &root.profiles {
                    if let Some(profile_bindings) = default_bindings.profiles.get(profile_name) {
                        let mut action_set_widgets = HashMap::new();

                        for subaction_path in profile.subaction_paths.iter() {
                            for (subpath, subpath_info) in profile.subpaths.iter() {
                                if let Some(side) = &subpath_info.side {
                                    if !subaction_path.ends_with(side) {
                                        continue;
                                    }
                                }

                                let binding_prefix = subaction_path.to_string().add(subpath); //e.g. /user/hand/left/input/select

                                for (set_name, set_bindings) in profile_bindings.action_sets.iter() {
                                    let set_info = application_info.action_sets.get(set_name).unwrap();
                                    let action_set_widget = if let Some(action_set_widget) = action_set_widgets.get_mut(set_name) {
                                        action_set_widget    
                                    } else {
                                        action_set_widgets.insert(set_name.clone(), ActionSetWidget(HashMap::new()));
                                        action_set_widgets.get_mut(set_name).unwrap()
                                    };
                                    let subaction_widget = if let Some(subaction_widget) = action_set_widget.0.get_mut(subaction_path) {
                                        subaction_widget    
                                    } else {
                                        action_set_widget.0.insert(subaction_path.clone(), SubactionWidget(HashMap::new()));
                                        action_set_widget.0.get_mut(subaction_path).unwrap()
                                    };

                                    for (action_name, action_bindings) in set_bindings.actions.iter() {
                                        let action_info = set_info.actions.get(action_name).unwrap();


                                        for binding in &action_bindings.bindings {
                                            for feature in &subpath_info.features {
                                                subaction_widget.0.insert(binding_prefix.clone().add("/").add(feature.to_str()), String::new());
                                            }

                                            if binding.starts_with(&binding_prefix) {
                                                let component = &binding[binding_prefix.len()..];
                                                if component.is_empty() {
                                                    let wanted = match &action_info.action_type {
                                                        ActionType::BooleanInput => [Feature::Value, Feature::Click].iter(),
                                                        ActionType::FloatInput => [Feature::Value].iter(),
                                                        ActionType::Vector2fInput => [Feature::Position].iter(),
                                                        ActionType::PoseInput => [Feature::Pose].iter(),
                                                        _ => [].iter()
                                                    };
                                                    for wanted_feature in wanted {
                                                        if subpath_info.features.contains(wanted_feature) {
                                                            if action_info.action_type != ActionType::Vector2fInput {
                                                                println!("{}/{} -> {}", &binding, wanted_feature.to_str(), &action_name);
                                                            } else {
                                                                println!("{} -> {}", &binding, &action_name);
                                                            }
                                                        }
                                                    }
                                                } else if subpath_info.features.contains(&Feature::from_str(&component[1..])) {
                                                    println!("{} -> {}", &binding, &action_name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        interaction_profile_widgets.insert(profile_name.clone(), InteractionProfileWidget(action_set_widgets));
                    }
                }
                self.input_states = interaction_profile_widgets;
                println!("{:#?}", self.input_states);
            },
            Message::UpdateText(string, idx) => {
                // self.input_values[idx] = string;
            },
            Message::Save => {},
            Message::None => (),
        }
        
        Command::none()
    }

    fn view<'a>(&mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new()
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh))
            .push(Button::new(&mut self.save_button_state, Text::new("Save")).on_press(Message::Save));

            // for (profile_name, action_sets) in &mut self.input_states {
            //     column = column.push(Text::new(profile_name).size(35));
            //     for (_, action_set_widget) in action_sets {
            //         column = column.push(Text::new(&action_set_widget.localized_name).size(30));
            //         for (_, aws) in &mut action_set_widget.actions {
            //             for bs in &mut aws.bindings {
            //                 let idx = bs.path_str_idx;
            //                 if let Some(subaction_path) = &bs.sub_action_path {
            //                     column = column.push(Row::new()
            //                         .push(Text::new(&aws.localized_name).size(30))
            //                         .push(Text::new("  ").size(30))
            //                         .push(Text::new(subaction_path).size(30))
            //                         .push(TextInput::new(&mut bs.input_state, "", self.input_values.get(idx).unwrap(), move |str| { Message::UpdateText(str, idx) }).size(30))
            //                     );
            //                 } else {
            //                     column = column.push(Row::new()
            //                         .push(Text::new(&aws.localized_name).size(30))
            //                         .push(Text::new("  ").size(30))
            //                         .push(TextInput::new(&mut bs.input_state, "", self.input_values.get(idx).unwrap(), move |str| { Message::UpdateText(str, idx) }).size(30))
            //                     );
            //                 }
            //             }    
            //         }
            //     }
            // }

        Scrollable::new(&mut self.scroll_state).push(column).into()
    }
}