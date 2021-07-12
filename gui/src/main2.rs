use std::{collections::HashMap, ops::Add, str::CharIndices};

use common::{application_bindings::*, interaction_profiles::Feature, serial::{self, CONFIG_DIR}, xrapplication_info::{ActionInfo, ActionType, XrApplicationInfo}};
use iced::{Application, Button, Column, Command, Element, PickList, Row, Scrollable, Text, TextInput, button, executor, pick_list, scrollable, text_input};

pub struct BindingsGUI {
    application_name: String,

    refresh_button_state: button::State,
    save_button_state: button::State,
    scroll_state: scrollable::State,

    selected_profile: String,
    profiles_pl_state: pick_list::State<String>,

    selected_set: String,
    set_pl_state: pick_list::State<String>,

    input_states: HashMap<String, InteractionProfileWidget>,
    input_values: Vec<String>,
    action_types: HashMap<String, HashMap<ActionType, Vec<String>>>
}

#[derive(Debug)]
struct InteractionProfileWidget(HashMap<String, ActionSetWidget>, String);
#[derive(Debug)]
struct ActionSetWidget(HashMap<String, SubactionWidget>, String);
#[derive(Debug)]
// struct SubactionWidget(HashMap<String, String>);
struct SubactionWidget(Vec<BindingWidget>);
#[derive(Debug)]
struct BindingWidget{
    subpath: String,
    action_type: ActionType,
    pick_list: pick_list::State<String>,
    input_value_idx: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    UpdateText(String, usize),
    SelectProfile(String),
    SelectActionSet(String),
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
                action_types: HashMap::new(),
                selected_profile: String::new(),
                profiles_pl_state: pick_list::State::default(),
                selected_set: String::new(),
                set_pl_state: pick_list::State::default(),
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

                self.action_types.clear();
                for (set_name, set_info) in application_info.action_sets.iter() {
                    let mut action_types_set = HashMap::new();
                    for action_type in &ActionType::all() {
                        let mut vec = set_info.actions.iter()
                        .filter_map(|(action_name, action_info)|
                            {
                                if *action_type == action_info.action_type || action_type.is_primitive() && action_info.action_type.is_primitive() {
                                    Some(action_name.to_owned())
                                } else {
                                    None
                                }
                            }
                        ).collect::<Vec<String>>();
                        vec.sort();
                        vec.push(String::new());
                        action_types_set.insert(action_type.to_owned(), vec);
                    }
                    self.action_types.insert(set_name.clone(), action_types_set);
                }
                
                let mut interaction_profile_widgets = HashMap::new();

                //TODO clean up this mess
                for (profile_name, profile) in &root.profiles {
                    let profile_bindings = default_bindings.profiles.get(profile_name);
                    if profile_bindings.is_none() { continue; }

                    let mut action_set_widgets = HashMap::new();
                    for (set_name, set_info) in &application_info.action_sets {
                        let set_bindings = if let Some(profile_bindings) = profile_bindings {
                            profile_bindings.action_sets.get(set_name)
                        } else { None };
                        println!("{}", set_name);
                        let mut subaction_widgets = HashMap::new();
                        for subaction_path in profile.subaction_paths.iter() {
                            println!("{}", subaction_path);
                            let mut binding_widgets = HashMap::new();
                            for (subpath, subpath_info) in profile.subpaths.iter() {
                                if let Some(side) = &subpath_info.side {
                                    if !subaction_path.ends_with(side) {
                                        continue;
                                    }
                                }
                                let binding_prefix = subaction_path.to_string().add(subpath); //e.g. /user/hand/left/input/select
                                let mut used_action_types = Vec::new();
                                for (action_name, action_info) in set_info.actions.iter() {
                                    used_action_types.push(action_info.action_type);

                                    if let Some(set_bindings) = set_bindings {
                                        if let Some(action_bindings) = set_bindings.actions.get(action_name) {
                                            for binding in &action_bindings.bindings {    
                                                if binding.starts_with(&binding_prefix) {
                                                    let component = &binding[binding_prefix.len()..];
                                                    if component.is_empty() { //Implicit component selected (or location in case of vector actions)
                                                        let wanted_features = match &action_info.action_type {
                                                            ActionType::BooleanInput => [Feature::Value, Feature::Click].iter(),
                                                            ActionType::FloatInput => [Feature::Value].iter(),
                                                            ActionType::Vector2fInput => [Feature::Position].iter(),
                                                            ActionType::PoseInput => [Feature::Pose].iter(),
                                                            _ => [].iter()
                                                        };
                                                        for wanted_feature in wanted_features {
                                                            if subpath_info.features.contains(wanted_feature) {
                                                                println!("{}/{} -> {}", &binding, wanted_feature.to_str(), &action_name);
                                                                binding_widgets.insert(String::from(&binding[subaction_path.len()..]).add("/").add(wanted_feature.to_str()), (wanted_feature.clone(), action_name.clone()));
                                                                continue;
                                                            }
                                                        }
                                                    } else if subpath_info.features.contains(&Feature::from_str(&component[1..])) {
                                                        println!("{} -> {}", &binding, &action_name);
                                                        binding_widgets.insert(String::from(&binding[subaction_path.len()..]), (Feature::from_str(&component[1..]) , action_name.clone()));
                                                    } else if action_info.action_type.is_primitive() && subpath_info.features.contains(&Feature::Position) { //Manually emulate x,y features if we have a position feature 
                                                        if component == "/x" || component == "/y" {
                                                            println!("{} -> {}", &binding, &action_name);
                                                            binding_widgets.insert(String::from(&binding[subaction_path.len()..]), (Feature::Value, action_name.clone()));
                                                        } 
                                                    }
                                                }
                                            }
                                        }
                                    };
                                }
                                for feature in &subpath_info.features { //Add all features which have no actions bound to them
                                    let mut inner = |key: String, feature: Feature| {
                                        if !binding_widgets.contains_key(&key) {
                                            binding_widgets.insert(key, (feature, String::new()));
                                        }
                                    };

                                    match feature {
                                        Feature::Click | Feature::Value => {
                                            if used_action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
                                                inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
                                            }
                                        },
                                        Feature::Position => {
                                            if used_action_types.contains(&ActionType::Vector2fInput) {
                                                inner(subpath.clone().add("/position"), feature.clone());
                                            }
                                            if used_action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
                                                inner(subpath.clone().add("/x"), Feature::Value);
                                                inner(subpath.clone().add("/y"), Feature::Value);
                                            }
                                        },
                                        _ => {
                                            if used_action_types.contains(&feature.get_type()) {
                                                inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            let mut sw = SubactionWidget(binding_widgets.into_iter().map(|(path, (feature, action))| -> BindingWidget {
                                self.input_values.push(action);
                                BindingWidget {
                                    subpath: path,
                                    pick_list: Default::default(),
                                    action_type: feature.get_type(),
                                    input_value_idx: self.input_values.len() - 1,
                                }
                            }).collect::<Vec<BindingWidget>>());
                            sw.0.sort_by_cached_key(|bw| {bw.subpath.clone()});
                            subaction_widgets.insert(subaction_path.clone(), sw);
                        }
                        action_set_widgets.insert(set_name.to_owned(), ActionSetWidget(subaction_widgets, set_info.localized_name.clone()));
                    }
                    interaction_profile_widgets.insert(profile_name.clone(), InteractionProfileWidget(action_set_widgets, profile.title.clone()));
                    
                }
                self.input_states = interaction_profile_widgets;
            },
            Message::UpdateText(string, idx) => {
                self.input_values[idx] = string;
            },
            Message::Save => {},
            Message::None => (),
            Message::SelectProfile(profile) => self.selected_profile = profile,
            Message::SelectActionSet(profile) => self.selected_set = profile,
        }
        
        Command::none()
    }

    fn view<'a>(&mut self) -> Element<'_, Self::Message> {
        println!("reload view");
        let mut column = Column::new()
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh))
            .push(Button::new(&mut self.save_button_state, Text::new("Save")).on_press(Message::Save));

            column = column.push(
                PickList::new(&mut self.profiles_pl_state, self.input_states.keys().map(|s| {s.to_owned()}).collect::<Vec<String>>(), Some(self.selected_profile.clone()), |profile| { Message::SelectProfile(profile) })
            );
            column = column.push(
                PickList::new(&mut self.set_pl_state, self.action_types.keys().map(|s| {s.to_owned()}).collect::<Vec<String>>(), Some(self.selected_set.clone()), |set| { Message::SelectActionSet(set) })
            );

            for (profile_name, profile_widget) in self.input_states.iter_mut() {
                if *profile_name != self.selected_profile { continue; }

                // column = column.push(Text::new(&profile_widget.1).size(35));
                for (set_name, set_widget) in profile_widget.0.iter_mut() {
                    if *set_name != self.selected_set { continue; }

                    let actions_for_action_type = &self.action_types[set_name];
                    // column = column.push(Text::new(&set_widget.1).size(32));
                    for (subaction_path, subaction_widget) in set_widget.0.iter_mut() {
                        column = column.push(Text::new(i18n(subaction_path)).size(30));
                        for binding_widget in subaction_widget.0.iter_mut() {
                            let idx = binding_widget.input_value_idx;
                            let pick_list = PickList::new(&mut binding_widget.pick_list, &actions_for_action_type[&binding_widget.action_type], Some(self.input_values[idx].clone()), move |f| { Message::UpdateText(f, idx) });
                            column = column.push(Row::new()
                                .push(Text::new(i18n(&binding_widget.subpath)).size(30))
                                .push(Text::new(" => ").size(30))
                                .push(pick_list)
                            );
                        }
                    }
                }
            }

        Scrollable::new(&mut self.scroll_state).push(column).into()
    }
}

fn i18n<'a>(path: &'a str) -> &'a str {
    match path {
        "/user/hand/right" => "Right Hand",
        "/user/hand/left" => "Left Hand",
        _ => path
    }
}