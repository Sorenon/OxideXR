use std::{collections::HashMap, env, ops::Add};

use common::{
    application_bindings::*,
    interaction_profiles::{Feature, InteractionProfile, Root},
    serial::{self, APPS_DIR},
    xrapplication_info::{ActionSetInfo, ActionType, XrApplicationInfo},
};
use iced::{
    button, executor,
    futures::{lock::Mutex, stream::Collect},
    pick_list, scrollable, text_input, Application, Button, Column, Command, Container, Element,
    Length, PickList, Row, Scrollable, Settings, Text, TextInput,
};

pub fn main() {
    let mut iter = env::args().into_iter();
    let mut app = None;
    while let Some(arg) = iter.next() {
        if arg == "-app" {
            app = iter.next();
            break;
        }
    }

    BindingsGUI::run(Settings::with_flags(app.unwrap())).unwrap();
}

pub struct BindingsGUI {
    application_name: String,

    refresh_button_state: button::State,
    save_button_state: button::State,
    scroll_state: scrollable::State,

    selected_profile: Option<String>,
    profiles_pick_list: pick_list::State<String>,

    selected_action_set: String,
    action_sets_pick_list: pick_list::State<String>,

    interaction_profiles: HashMap<String, InteractionProfileWidget>,

    action_sets_data: HashMap<String, ActionSetData>,

    input_values: Vec<String>,
}

#[derive(Debug)]
struct ActionSetData {
    name: String,
    actions_for_types: HashMap<ActionType, Vec<String>>,
    action_names: HashMap<String, String>,
}

#[derive(Debug)]
struct InteractionProfileWidget {
    name: String,
    action_sets: HashMap<String, ActionSetWidget>,
}

#[derive(Debug)]
struct ActionSetWidget(HashMap<String /* subaction_path */, DeviceWidget>);

#[derive(Debug)]
struct DeviceWidget {
    sub_devices: Vec<SubDeviceWidget>,
}

#[derive(Debug)]
struct SubDeviceWidget {
    localized_name: String,
    name: String,
    components: HashMap<String, ComponentWidget>,
}

#[derive(Debug)]
struct ComponentWidget {
    action_type: ActionType,
    pick_list: pick_list::State<String>,
    input_value_idx: usize,
}

impl ComponentWidget {
    fn new(action_type: ActionType, input_values: &mut Vec<String>) -> Self {
        input_values.push(String::new());
        Self {
            action_type,
            pick_list: pick_list::State::default(),
            input_value_idx: input_values.len() - 1,
        }
    }
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
                action_sets_data: HashMap::new(),
                selected_profile: None,
                profiles_pick_list: pick_list::State::default(),
                selected_action_set: String::new(),
                action_sets_pick_list: pick_list::State::default(),
                application_name: flags,
            },
            Command::none(),
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
                let file_path = format!("{}{}/actions.json", APPS_DIR, uuid);
                let application_info = serial::read_json::<XrApplicationInfo>(&file_path).unwrap();
                let file_path = format!("{}{}/default_bindings.json", APPS_DIR, uuid);
                let default_bindings =
                    serial::read_json::<ApplicationBindings>(&file_path).unwrap();

                let root = common::interaction_profiles::generate();

                self.action_sets_data = application_info
                    .action_sets
                    .iter()
                    .map(|(set_name, set_info)| {
                        (
                            set_info.localized_name.clone(),
                            ActionSetData::new(set_name.clone(), set_info),
                        )
                    })
                    .collect::<HashMap<_, _>>();

                let mut input_values = Vec::new();

                self.interaction_profiles = root
                    .profiles
                    .into_iter()
                    .map(|(profile_name, profile_info)| {
                        let profile_bindings = default_bindings.profiles.get(&profile_name);

                        let child_action_sets = application_info
                            .action_sets
                            .iter()
                            .map(|(set_name, set_info)| {
                                let set_bindings = if let Some(profile_bindings) = profile_bindings
                                {
                                    profile_bindings.action_sets.get(set_name)
                                } else {
                                    None
                                };
                                let action_types = self.action_sets_data[&set_info.localized_name]
                                    .actions_for_types
                                    .iter()
                                    .filter_map(|(t, v)| if v.len() > 1 { Some(t) } else { None })
                                    .collect();

                                let subaction_widgets = profile_info
                                    .subaction_paths
                                    .iter()
                                    .map(|subaction_path| {
                                        (
                                            subaction_path.clone(),
                                            load_device_for_action_set(
                                                subaction_path,
                                                &profile_info,
                                                set_info,
                                                set_bindings,
                                                &action_types,
                                                &mut input_values,
                                            ),
                                        )
                                    })
                                    .collect();
                                (
                                    set_info.localized_name.clone(),
                                    ActionSetWidget(subaction_widgets),
                                )
                            })
                            .collect();

                        (
                            profile_info.title,
                            InteractionProfileWidget {
                                name: profile_name,
                                action_sets: child_action_sets,
                            },
                        )
                    })
                    .collect();

                self.input_values = input_values;
            }
            Message::UpdateText(string, idx) => {
                self.input_values[idx] = string;
            }
            Message::Save => {
                let mut profiles = HashMap::new();

                for (_, profile_widget) in self.interaction_profiles.iter() {
                    let profile_name = &profile_widget.name;
                    let mut action_sets = HashMap::new();

                    for (action_set_localized, action_set_widget) in
                        profile_widget.action_sets.iter()
                    {
                        let action_set_data = &self.action_sets_data[action_set_localized];

                        let mut actions = HashMap::new();

                        for (subaction_path, device_widget) in action_set_widget.0.iter() {

                            for sub_device_widget in device_widget.sub_devices.iter() {
                                for (component_name, component_widget) in
                                    sub_device_widget.components.iter()
                                {
                                    let action_localized_name =
                                        &self.input_values[component_widget.input_value_idx];

                                    if !action_localized_name.is_empty() {
                                        let component_name = if component_name == "position" {
                                            String::new()
                                        } else {
                                            format!("/{}", component_name)
                                        };

                                        let action: &mut ActionBindings =
                                            actions.entry(action_set_data.action_names[action_localized_name].clone()).or_default();
                                        action.bindings.push(format!(
                                            "{}{}{}",
                                            subaction_path, sub_device_widget.name, &component_name,
                                        ));
                                    }
                                }
                            }
                        }

                        if !actions.is_empty() {
                            action_sets
                                .insert(action_set_data.name.clone(), ActionSetBindings { actions });
                        }
                    }

                    if !action_sets.is_empty() {
                        profiles.insert(
                            profile_name.clone(),
                            InteractionProfileBindings { action_sets },
                        );
                    }
                }

                let bindings = ApplicationBindings { profiles };
                println!("{}", serde_json::to_string(&bindings).unwrap());
            }
            Message::None => (),
            Message::SelectProfile(profile) => self.selected_profile = Some(profile),
            Message::SelectActionSet(profile) => self.selected_action_set = profile,
        }

        Command::none()
    }

    fn view<'a>(&'a mut self) -> Element<'_, Self::Message> {
        let mut column = Column::new()
            .spacing(10)
            .push(
                Button::new(&mut self.refresh_button_state, Text::new("Reload"))
                    .on_press(Message::Refresh),
            )
            .push(
                Button::new(&mut self.save_button_state, Text::new("Apply"))
                    .on_press(Message::Save),
            )
            .push(PickList::new(
                &mut self.profiles_pick_list,
                self.interaction_profiles
                    .keys()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>(),
                self.selected_profile.clone(),
                |profile| Message::SelectProfile(profile),
            ))
            .push(PickList::new(
                &mut self.action_sets_pick_list,
                self.action_sets_data
                    .keys()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>(),
                Some(self.selected_action_set.clone()),
                |set| Message::SelectActionSet(set),
            ));

        if let Some(selected_profile) = &self.selected_profile {
            let profile_widget = self.interaction_profiles.get_mut(selected_profile).unwrap();
            let selected_set = &self.selected_action_set;
            if let Some(set_widget) = profile_widget.action_sets.get_mut(selected_set) {
                let actions_for_types = &self.action_sets_data[selected_set].actions_for_types;

                let mut device_columns = Row::new().spacing(50);

                for (subaction_path, device_widget) in set_widget.0.iter_mut() {
                    let mut device_column = Column::new().spacing(5);

                    device_column =
                        device_column.push(Text::new(localize_subaction_path(subaction_path)).size(30));
                    for sub_device in device_widget.sub_devices.iter_mut() {
                        device_column =
                            device_column.push(Text::new(&sub_device.localized_name).size(30));

                        for (component_name, component_widget) in sub_device.components.iter_mut() {
                            let idx = component_widget.input_value_idx;

                            device_column = device_column.push(
                                Row::new()
                                    .push(Text::new("   ").size(30))
                                    .push(Text::new(component_name).size(30))
                                    .push(Text::new(": ").size(30))
                                    .push(PickList::new(
                                        &mut component_widget.pick_list,
                                        &actions_for_types[&component_widget.action_type],
                                        Some(self.input_values[idx].clone()),
                                        move |s| Message::UpdateText(s, idx),
                                    )),
                            );
                        }
                    }

                    device_columns = device_columns.push(device_column);
                }
                column = column.push(device_columns);
            }
        }

        Scrollable::new(&mut self.scroll_state)
            .padding(20)
            .push(Container::new(column).width(Length::Fill))
            .into()
    }
}

fn load_device_for_action_set(
    subaction_path: &str,
    profile_info: &InteractionProfile,
    set_info: &ActionSetInfo,
    set_bindings: Option<&ActionSetBindings>,
    action_types: &Vec<&ActionType>,
    input_values: &mut Vec<String>,
) -> DeviceWidget {
    let mut sub_devices = Vec::new();
    for (subpath, sub_device_info) in profile_info.subpaths.iter() {
        //If this subpath only exists on a certain subaction path we skip it (e.g. /input/x/click would be skipped for /user/hand/right in interaction_profiles/oculus/touch_controller)
        if let Some(side) = &sub_device_info.side {
            if !subaction_path.ends_with(side) {
                continue;
            }
        }

        let mut components = HashMap::new();

        for feature in &sub_device_info.features {
            match feature {
                Feature::Click | Feature::Value => {
                    if action_types
                        .iter()
                        .find(|action_type| action_type.is_primitive())
                        .is_some()
                    {
                        components.insert(
                            feature.to_str().to_owned(),
                            ComponentWidget::new(feature.get_type(), input_values),
                        );
                    }
                }
                Feature::Position => {
                    if action_types.contains(&&ActionType::Vector2fInput) {
                        components.insert(
                            "position".to_owned(),
                            ComponentWidget::new(ActionType::Vector2fInput, input_values),
                        );
                    }
                    if action_types
                        .iter()
                        .find(|action_type| action_type.is_primitive())
                        .is_some()
                    {
                        components.insert(
                            "x".to_owned(),
                            ComponentWidget::new(ActionType::FloatInput, input_values),
                        );
                        components.insert(
                            "y".to_owned(),
                            ComponentWidget::new(ActionType::FloatInput, input_values),
                        );
                    }
                }
                _ => {
                    if action_types.contains(&&feature.get_type()) {
                        components.insert(
                            feature.to_str().to_owned(),
                            ComponentWidget::new(feature.get_type(), input_values),
                        );
                    }
                }
            }
        }

        //Don't bother adding this subpath if there are no features matching the action types in the parent action set
        if components.is_empty() {
            continue;
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
                            if component.is_empty() {
                                //Implicit component selected (or location in case of vector actions)
                                let wanted_features = match &action_info.action_type {
                                    ActionType::BooleanInput => {
                                        [Feature::Value, Feature::Click].iter()
                                    }
                                    ActionType::FloatInput => [Feature::Value].iter(),
                                    ActionType::Vector2fInput => [Feature::Position].iter(),
                                    ActionType::PoseInput => [Feature::Pose].iter(),
                                    _ => [].iter(),
                                };
                                for wanted_feature in wanted_features {
                                    if sub_device_info.features.contains(wanted_feature) {
                                        input_values
                                            [components[wanted_feature.to_str()].input_value_idx] =
                                            action_info.localized_name.clone();
                                        continue;
                                    }
                                }
                            } else if sub_device_info
                                .features
                                .contains(&Feature::from_str(&component[1..]))
                            {
                                input_values[components[&component[1..]].input_value_idx] =
                                    action_info.localized_name.clone();
                            } else if action_info.action_type.is_primitive()
                                && sub_device_info.features.contains(&Feature::Position)
                            {
                                //Manually emulate x,y features if we have a position feature
                                if component == "/x" || component == "/y" {
                                    input_values[components[&component[1..]].input_value_idx] =
                                        action_info.localized_name.clone();
                                }
                            } else {
                                panic!()
                            }
                        }
                    }
                }
            };
        }

        sub_devices.push(SubDeviceWidget {
            localized_name: sub_device_info.localized_name.clone(),
            name: subpath.clone(),
            components,
        });
    }

    sub_devices.sort_by_cached_key(|sub_dev| sub_dev.localized_name.clone());
    DeviceWidget { sub_devices }
}

fn localize_subaction_path<'a>(path: &'a str) -> &'a str {
    match path {
        "/user/hand/right" => "Right Hand",
        "/user/hand/left" => "Left Hand",
        _ => path,
    }
}

impl ActionSetData {
    fn new(set_name: String, set_info: &ActionSetInfo) -> Self {
        let actions_for_types = ActionType::all()
            .iter()
            .map(|action_type| {
                let mut vec = set_info
                    .actions
                    .iter()
                    .filter_map(|(_, action_info)| {
                        if *action_type == action_info.action_type
                            || action_type.is_primitive() && action_info.action_type.is_primitive()
                        {
                            Some(action_info.localized_name.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>();
                vec.sort();
                vec.push(String::new());
                (*action_type, vec)
            })
            .collect::<HashMap<_, _>>();

        ActionSetData {
            name: set_name,
            actions_for_types,
            action_names: set_info
                .actions
                .iter()
                .map(|(action_name, action_info)| {
                    (action_info.localized_name.clone(), action_name.clone())
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}
