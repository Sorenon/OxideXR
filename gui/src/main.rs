use std::{collections::HashMap, env, ops::Add};

use common::{application_bindings::*, interaction_profiles::{Feature, InteractionProfile}, serial::{self, CONFIG_DIR}, xrapplication_info::{ActionSetInfo, ActionType, XrApplicationInfo}};
use iced::{Application, Button, Column, Command, Container, Element, Length, PickList, Row, Scrollable, Settings, Text, TextInput, button, executor, futures::lock::Mutex, pick_list, scrollable, text_input};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    BindingsGUI::run(Settings::with_flags(args.get(1).unwrap().clone())).unwrap();
}

pub struct BindingsGUI {
    application_name: String,

    refresh_button_state: button::State,
    save_button_state: button::State,
    scroll_state: scrollable::State,

    selected_profile: Option<String>,
    profiles_state: pick_list::State<String>,

    selected_action_set: String,
    action_sets_state: pick_list::State<String>,

    interaction_profiles: HashMap<String, InteractionProfileGUI>,
    input_values: Vec<String>,
    action_sets: HashMap<String, ActionSetGUI>
}

#[derive(Debug)]
struct ActionSetGUI {
    delocalized_name: String,
    actions_for_types: HashMap<ActionType, Vec<String>>,
}

#[derive(Debug)]
struct InteractionProfileGUI {
    delocalized_name: String,
    action_sets: HashMap<String, ActionSetWidget>
}

#[derive(Debug)]
struct ActionSetWidget(HashMap<String, SubactionWidget>, String);

#[derive(Debug)]
struct SubactionWidget(Vec<SubpathGUI>);

#[derive(Debug)]
struct SubpathGUI { 
    localized_name: String, 
    name: String,
    components: HashMap<String, ComponentGUI>,   
}

#[derive(Debug)]
struct ComponentGUI {
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
    #[allow(dead_code)]
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
                interaction_profiles: HashMap::new(),
                input_values: Vec::new(),
                action_sets: HashMap::new(),
                selected_profile: None,
                profiles_state: pick_list::State::default(),
                selected_action_set: String::new(),
                action_sets_state: pick_list::State::default(),
                application_name: flags,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from(format!("OxideXR for {}", &self.application_name))
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

                self.action_sets.clear();
                for (set_name, set_info) in application_info.action_sets.iter() {
                    let mut action_types_set = HashMap::new();
                    for action_type in &ActionType::all() {
                        let mut vec = set_info.actions.iter()
                        .filter_map(|(_, action_info)| {
                                if *action_type == action_info.action_type || action_type.is_primitive() && action_info.action_type.is_primitive() {
                                    Some(action_info.localized_name.to_owned())
                                } else {
                                    None
                                }
                            }
                        ).collect::<Vec<String>>();
                        vec.sort();
                        vec.push(String::new());
                        action_types_set.insert(action_type.to_owned(), vec);
                    }
                    self.action_sets.insert(set_info.localized_name.clone(), ActionSetGUI {
                        delocalized_name: set_name.clone(),
                        actions_for_types: action_types_set,
                    });
                }
                
                self.interaction_profiles = root.profiles.into_iter()
                .map(|(profile_name, profile_info)| {
                    let profile_bindings = default_bindings.profiles.get(&profile_name);

                    let child_action_sets = application_info.action_sets.iter()
                    .map(|(set_name, set_info)| {
                        let set_bindings = if let Some(profile_bindings) = profile_bindings {
                            profile_bindings.action_sets.get(set_name)
                        } else { None };
                        let action_types = self.action_sets[&set_info.localized_name].actions_for_types.iter().filter_map(|(t, v)| {
                            if v.len() > 1 { Some(t) } else { None }
                        }).collect();

                        let subaction_widgets = profile_info.subaction_paths.iter()
                        .map(|subaction_path| {
                            (subaction_path.clone(), load_subaction_path_for_set(subaction_path, &profile_info, set_info, set_bindings, &action_types))
                        }).collect();
                        (set_info.localized_name.clone(), ActionSetWidget(subaction_widgets, set_info.localized_name.clone()))
                    }).collect();

                    (profile_info.title, InteractionProfileGUI {
                        delocalized_name: profile_name,
                        action_sets: child_action_sets
                    })    
                }).collect();
            },
            Message::UpdateText(string, idx) => {
                self.input_values[idx] = string;
            },
            Message::Save => {},
            Message::None => (),
            Message::SelectProfile(profile) => {
                self.selected_profile = Some(profile)
            },
            Message::SelectActionSet(profile) => self.selected_action_set = profile,
        }
        
        Command::none()
    }

    fn view<'a>(&'a mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new().spacing(10)
            .push(Button::new(&mut self.refresh_button_state, Text::new("Reload")).on_press(Message::Refresh))
            .push(Button::new(&mut self.save_button_state, Text::new("Save")).on_press(Message::Save))
            .push(
                PickList::new(&mut self.profiles_state, self.interaction_profiles.keys().map(|s| {s.to_owned()}).collect::<Vec<String>>(), self.selected_profile.clone(), |profile| { Message::SelectProfile(profile) })
            ).push(
                PickList::new(&mut self.action_sets_state, self.action_sets.keys().map(|s| {s.to_owned()}).collect::<Vec<String>>(), Some(self.selected_action_set.clone()), |set| { Message::SelectActionSet(set) })
            );

            if let Some(pp) = &self.selected_profile {
                let profile_widget = self.interaction_profiles.get_mut(pp).unwrap();
                let selected_set = &self.selected_action_set;
                if let Some(set_widget) = profile_widget.action_sets.get_mut(selected_set) {
                    let set_wg_info = &self.action_sets[selected_set].actions_for_types;
                    for (subaction_path, subaction_widget) in set_widget.0.iter_mut() {
                        column = column.push(Text::new(localize(subaction_path)).size(30));
                        for subpath_gui in subaction_widget.0.iter() {
                            column = column.push(Text::new(&subpath_gui.localized_name));

                            // let idx = binding_widget.input_value_idx;
                            // let pick_list = PickList::new(&mut binding_widget.pick_list, &set_wg_info[&binding_widget.action_type], Some(self.input_values[idx].clone()), move |f| { Message::UpdateText(f, idx) });
                            // column = column.push(Row::new()
                            //     .push(Text::new(localize(&binding_widget.subpath)).size(30))
                            //     .push(Text::new(" => ").size(30))
                            //     .push(pick_list)
                            // );
                        }
                    }
                }
            }

            Scrollable::new(&mut self.scroll_state).padding(20).push(Container::new(column).width(Length::Fill)).into()
    }
}


fn load_subaction_path_for_set(subaction_path: &str, profile_info: &InteractionProfile, set_info: &ActionSetInfo, set_bindings: Option<&ActionSetBindings>, action_types: &Vec<&ActionType>) -> SubactionWidget {
    // let mut binding_widgets = HashMap::new();

    let mut subpaths = Vec::new();
    for (subpath, subpath_info) in profile_info.subpaths.iter() {
        //If this subpath only exists on a certain subaction path we skip it (e.g. /input/x/click would be skipped for /user/hand/right in interaction_profiles/oculus/touch_controller)
        if let Some(side) = &subpath_info.side {
            if !subaction_path.ends_with(side) {
                continue;
            }
        }

        let mut components = Vec::new();

        for feature in &subpath_info.features {
            match feature {
                Feature::Click | Feature::Value => {
                    if action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
                        components.push(feature.to_str());
                        // inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
                    }
                },
                Feature::Position => {
                    if action_types.contains(&&ActionType::Vector2fInput) {
                        components.push("position");

                        // inner(subpath.clone().add("/position"), feature.clone());
                    }
                    if action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
                        components.push("x");
                        components.push("y");

                        // inner(subpath.clone().add("/x"), Feature::Value);
                        // inner(subpath.clone().add("/y"), Feature::Value);
                    }
                },
                _ => {
                    if action_types.contains(&&feature.get_type()) {
                        components.push(feature.to_str());
                        // inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
                    }
                }
            }
        }

        //Don't bother adding this subpath if there are no features matching the action types in the parent action set
        if components.is_empty() {
            continue;
        }

        subpaths.push(SubpathGUI {
            localized_name: subpath_info.localized_name.clone().add(&format!("{:?}", &action_types)),
            name: subpath.clone(),
            components: HashMap::new(),
        });

        // let binding_prefix = subaction_path.to_string().add(subpath); //e.g. /user/hand/left/input/select
        // let mut used_action_types = Vec::new();
        // for (action_name, action_info) in set_info.actions.iter() {
        //     used_action_types.push(action_info.action_type);

        //     if let Some(set_bindings) = set_bindings {
        //         if let Some(action_bindings) = set_bindings.actions.get(action_name) {
        //             for binding in &action_bindings.bindings {    
        //                 if binding.starts_with(&binding_prefix) {
        //                     let component = &binding[binding_prefix.len()..];
        //                     if component.is_empty() { //Implicit component selected (or location in case of vector actions)
        //                         let wanted_features = match &action_info.action_type {
        //                             ActionType::BooleanInput => [Feature::Value, Feature::Click].iter(),
        //                             ActionType::FloatInput => [Feature::Value].iter(),
        //                             ActionType::Vector2fInput => [Feature::Position].iter(),
        //                             ActionType::PoseInput => [Feature::Pose].iter(),
        //                             _ => [].iter()
        //                         };
        //                         for wanted_feature in wanted_features {
        //                             if subpath_info.features.contains(wanted_feature) {
        //                                 println!("{}/{} -> {}", &binding, wanted_feature.to_str(), &action_name);
        //                                 binding_widgets.insert(String::from(&binding[subaction_path.len()..]).add("/").add(wanted_feature.to_str()), (wanted_feature.clone(), action_info.localized_name.clone()));
        //                                 continue;
        //                             }
        //                         }
        //                     } else if subpath_info.features.contains(&Feature::from_str(&component[1..])) {
        //                         println!("{} -> {}", &binding, &action_name);
        //                         binding_widgets.insert(String::from(&binding[subaction_path.len()..]), (Feature::from_str(&component[1..]) , action_info.localized_name.clone()));
        //                     } else if action_info.action_type.is_primitive() && subpath_info.features.contains(&Feature::Position) { //Manually emulate x,y features if we have a position feature 
        //                         if component == "/x" || component == "/y" {
        //                             println!("{} -> {}", &binding, &action_name);
        //                             binding_widgets.insert(String::from(&binding[subaction_path.len()..]), (Feature::Value, action_info.localized_name.clone()));
        //                         } 
        //                     }
        //                 }
        //             }
        //         }
        //     };
        // }

        // for feature in &subpath_info.features { //Add all features which have no actions bound to them
        //     let mut inner = |key: String, feature: Feature| {
        //         if !binding_widgets.contains_key(&key) {
        //             binding_widgets.insert(key, (feature, String::new()));
        //         }
        //     };

        //     match feature {
        //         Feature::Click | Feature::Value => {
        //             if used_action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
        //                 inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
        //             }
        //         },
        //         Feature::Position => {
        //             if used_action_types.contains(&ActionType::Vector2fInput) {
        //                 inner(subpath.clone().add("/position"), feature.clone());
        //             }
        //             if used_action_types.iter().find(|action_type| {action_type.is_primitive()}).is_some() {
        //                 inner(subpath.clone().add("/x"), Feature::Value);
        //                 inner(subpath.clone().add("/y"), Feature::Value);
        //             }
        //         },
        //         _ => {
        //             if used_action_types.contains(&feature.get_type()) {
        //                 inner(subpath.clone().add("/").add(feature.to_str()), feature.clone());
        //             }
        //         }
        //     }
        // }
    }
    // let mut sw = SubactionWidget(binding_widgets.into_iter().map(|(path, (feature, action))| -> BindingGUI {
    //     self.input_values.push(action);
    //     BindingGUI {
    //         subpath: path,
    //         pick_list: Default::default(),
    //         action_type: feature.get_type(),
    //         input_value_idx: self.input_values.len() - 1,
    //     }
    // }).collect::<Vec<BindingGUI>>());
    // sw.0.sort_by_cached_key(|bw| {bw.subpath.clone()});
    // subaction_widgets.insert(subaction_path.clone(), sw);

    SubactionWidget(subpaths)
}


fn localize<'a>(path: &'a str) -> &'a str {
    match path {
        "/user/hand/right" => "Right Hand",
        "/user/hand/left" => "Left Hand",
        _ => path
    }
}