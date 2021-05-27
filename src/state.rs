use std::rc::Rc;

use reactive_state::{Reducer, ReducerResult, StoreRef};

use crate::{
    history::History, i18n::LocalizedString, parsers::scenery_packs::SceneryPack,
    settings::Settings,
};

#[derive(Clone)]
pub struct ActionHistoryItem<T> {
    pub label: LocalizedString,
    pub item: T,
}

impl<T: Default> ActionHistoryItem<T> {
    fn with_empty_label(item: T) -> Self {
        Self {
            label: LocalizedString::new(|| String::new()),
            item,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ActionHistoryItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionHistoryItem")
            .field("item", &self.item)
            .finish()
    }
}

impl<T: Default> Default for ActionHistoryItem<T> {
    fn default() -> Self {
        Self::with_empty_label(Default::default())
    }
}

#[derive(Clone, Default)]
pub struct SceneryPacksHistoryItem {
    pub scenery_packs: im_rc::Vector<SceneryPack>,
    pub id: u64,
}

impl From<im_rc::Vector<SceneryPack>> for SceneryPacksHistoryItem {
    fn from(scenery_packs: im_rc::Vector<SceneryPack>) -> Self {
        Self {
            scenery_packs,
            id: rand::random(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ScenableState {
    pub settings: Rc<Settings>,
    pub scenery_packs: im_rc::Vector<SceneryPack>,
    pub scenery_packs_history: History<ActionHistoryItem<SceneryPacksHistoryItem>>,
    /// The id of the [SceneryPacksHistoryItem] which corresponds to
    /// the currently saved state of the `scenery_packs.ini` file.
    pub scenery_packs_saved_history_id: u64,
}

impl std::fmt::Debug for ScenableState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.settings.fmt(f)?;
        Ok(())
    }
}

impl ScenableState {
    fn senery_packs_current_history_id(&self) -> u64 {
        let (current_history, _) = self.scenery_packs_history.peek_current();
        current_history.item.id
    }

    /// Returns whether or not the state of scenery packs is synchronized with what is on disk.
    pub fn scenery_packs_synchronized(&self) -> bool {
        self.scenery_packs_saved_history_id == self.senery_packs_current_history_id()
    }
}

pub enum ActionHistory {
    Some(LocalizedString),
    None,
}

impl ActionHistory {
    fn format_label(&self) -> Option<String> {
        match self {
            ActionHistory::Some(label) => Some(label.to_string()),
            ActionHistory::None => None,
        }
    }
}

impl std::fmt::Debug for ActionHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionHistory::Some(_) => f.debug_tuple("Some").finish(),
            ActionHistory::None => f.debug_tuple("None").finish(),
        }
    }
}

/// Overwrite/update the state of all scenery packs in memory.
#[derive(Debug)]
pub struct UpdateSceneryPacks {
    pub scenery_packs: im_rc::Vector<SceneryPack>,
    pub history: ActionHistory,
    pub reset_history: bool,
}

/// Edit a single scenery pack.
#[derive(Debug)]
pub struct UpdateSceneryPack {
    pub index: usize,
    pub scenery_pack: SceneryPack,
    pub history: ActionHistory,
}

pub enum ScenableAction {
    /// Update the application [Settings].
    UpdateSettings(Settings),
    /// See [UpdateSceneryPacks].
    UpdateSceneryPacks(UpdateSceneryPacks),
    /// See [UpdateSceneryPack].
    UpdateSceneryPack(UpdateSceneryPack),
    /// Undo previous change to scenery packs.
    UndoSceneryPacks,
    /// Redo previous change to scenery packs.
    RedoSceneryPacks,
    /// Notifies that the state of scenery packs has been read from or
    /// written to disk.
    UpdateSceneryPacksSyncStatus,
}

impl std::fmt::Debug for ScenableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenableAction::UpdateSettings(_) => f.debug_tuple("UpdateSettings").finish(),
            ScenableAction::UpdateSceneryPacks(_) => f.debug_tuple("UpdateScenery").finish(),
            ScenableAction::UpdateSceneryPack(action) => f
                .debug_tuple("UpdateScenery")
                .field(&action.index)
                .field(&action.history.format_label())
                .finish(),
            ScenableAction::UndoSceneryPacks => f.debug_tuple("UndoSceneryPacks").finish(),
            ScenableAction::RedoSceneryPacks => f.debug_tuple("RedoSceneryPacks").finish(),
            ScenableAction::UpdateSceneryPacksSyncStatus => {
                f.debug_tuple("UpdateSceneryPacksSyncStatus").finish()
            }
        }
    }
}

pub type ScenableStateRef = StoreRef<ScenableState, ScenableAction, (), ()>;

pub struct ScenableReducer;

impl Reducer<ScenableState, ScenableAction, (), ()> for ScenableReducer {
    #[tracing::instrument(skip(self, prev_state))]
    fn reduce(
        &self,
        prev_state: &Rc<ScenableState>,
        action: &ScenableAction,
    ) -> ReducerResult<ScenableState, (), ()> {
        let mut new_state = (**prev_state).clone();
        match action {
            ScenableAction::UpdateSettings(new_settings) => {
                new_state.settings = Rc::new(new_settings.clone());

                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
            ScenableAction::UpdateSceneryPacks(action) => {
                match &action.history {
                    ActionHistory::Some(label) => {
                        let history_item = ActionHistoryItem {
                            label: label.clone(),
                            item: From::from(action.scenery_packs.clone()),
                        };

                        if action.reset_history {
                            new_state.scenery_packs_history.reset(history_item);
                        } else {
                            new_state.scenery_packs_history.push(history_item);
                        }
                    }
                    ActionHistory::None => {}
                }

                new_state.scenery_packs = action.scenery_packs.clone();

                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
            ScenableAction::UpdateSceneryPack(action) => {
                if let Some(scenery_pack) = new_state.scenery_packs.get_mut(action.index) {
                    *scenery_pack = action.scenery_pack.clone();
                } else {
                    tracing::error!("No scenery pack exists for the specified index")
                }

                match &action.history {
                    ActionHistory::Some(label) => {
                        new_state.scenery_packs_history.push(ActionHistoryItem {
                            label: label.clone(),
                            item: From::from(new_state.scenery_packs.clone()),
                        });
                    }
                    ActionHistory::None => {}
                }

                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
            ScenableAction::UndoSceneryPacks => {
                if let Some((history_item, _)) = new_state.scenery_packs_history.undo() {
                    new_state.scenery_packs = history_item.item.scenery_packs.clone();
                } else {
                    tracing::warn!("Can't undo, history is either empty or this is the first item");
                }
                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
            ScenableAction::RedoSceneryPacks => {
                if let Some((history_item, _)) = new_state.scenery_packs_history.redo() {
                    new_state.scenery_packs = history_item.item.scenery_packs.clone();
                } else {
                    tracing::warn!("Can't redo, history is either empty or this is the first item");
                }
                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
            ScenableAction::UpdateSceneryPacksSyncStatus => {
                new_state.scenery_packs_saved_history_id =
                    new_state.senery_packs_current_history_id();
                ReducerResult {
                    state: Rc::new(new_state),
                    events: vec![],
                    effects: vec![],
                }
            }
        }
    }
}
