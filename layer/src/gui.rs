use std::{cell::RefCell, collections::HashMap, sync::Weak};

use common::{application_bindings::{ActionSetBindings, ApplicationBindings}, serial::{self, CONFIG_DIR}, xrapplication_info::{ActionInfo, XrApplicationInfo}};
use iced::{Application, Button, Column, Command, Element, Row, Scrollable, Settings, Text, TextInput, button, executor, scrollable, text_input};

use crate::wrappers::*;

pub fn do_thing(flag: XrApplicationInfo) {
    BindingsGUI::run_any_thread(Settings::with_flags(flag)).unwrap();
}

pub struct BindingsGUI {
    application_info: XrApplicationInfo,
    default_bindings: ApplicationBindings,

    refresh_button_state: button::State,
    scroll_state: scrollable::State,

    input_states: Vec<RefCell<text_input::State>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Refresh,
}

impl Application for BindingsGUI {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = XrApplicationInfo;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            BindingsGUI {
                refresh_button_state: button::State::new(),
                scroll_state: scrollable::State::new(),
                input_states: Vec::new(),
                application_info: flags,
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
            },
        }
        
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new()
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh));

            self.input_states.clear();

        for (profile_name, profile_binding) in &self.default_bindings.profiles {
            column = column.push(Text::new(profile_name).size(35));

            for (set_name, set_info) in &self.application_info.action_sets {
                column = column.push(Text::new(&set_info.localized_name).size(30));
                
                if let Some(set_bindings) = profile_binding.action_sets.get(set_name) {
                    for (action_name, action_info) in &set_info.actions {
                        {self.input_states.push(RefCell::new(text_input::State::new()));}
                        let len = self.input_states.len() - 1;
                        let input_state = self.input_states.get(len).unwrap().get_mut();

                        match set_bindings.actions.get(action_name) {
                            Some(action_bindings) => column = column.push(Row::new()
                                .push(Text::new(&action_info.localized_name).size(25))
                                .push(Text::new(" -> ").size(25))
                                .push(Text::new(serde_json::to_string(&action_bindings.binding).unwrap()).size(25))
                            ),
                            None => column = column.push(Row::new() 
                                .push(Text::new(&action_info.localized_name).size(25))
                                .push(TextInput::new(input_state, "", "", |str| -> Self::Message { Message::Refresh }).size(25))
                            ),
                        }
                    }
                }
                else {
                    for (_, action_info) in &set_info.actions {
                        column = column.push(Row::new() 
                                .push(Text::new(&action_info.localized_name).size(25))
                                .push(Text::new(" -> X").size(25))
                            );
                    }
                }
            }
        }

        let json = serde_json::to_string_pretty(&self.application_info).unwrap();
        let column = column.push(Text::new(json));
        let json = serde_json::to_string_pretty(&self.default_bindings).unwrap();
        let column = column.push(Text::new(json));

        Scrollable::new(&mut self.scroll_state).push(column).into()
    }
}