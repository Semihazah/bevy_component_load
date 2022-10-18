use std::marker::PhantomData;

use bevy::{
    ecs::{component::Component, system::Command},
    prelude::*,
    reflect::FromReflect,
    utils::HashSet,
};
use bevy_trait_query::{impl_trait_query, All, RegisterExt};

#[cfg(feature = "asset_tracking")]
use iyes_progress::prelude::AssetsLoading;

#[cfg(test)]
mod test;

pub struct BevyComponentLoadPlugin;

impl Plugin for BevyComponentLoadPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsLoaded>()
        .add_system_to_stage(CoreStage::PostUpdate, load_data)
        .add_system_to_stage(CoreStage::PostUpdate, unload_data)
        ;
    }
}

#[derive(Reflect, FromReflect, Default, Debug, Component)]
#[reflect(Component)]
pub struct IsLoaded;

pub trait Loadable: 'static {
    fn load_data(
        &mut self,
        asset_server: &Res<AssetServer>,

        #[cfg(feature = "asset_tracking")]
        loading: &mut ResMut<AssetsLoading>,
    ) -> anyhow::Result<()>;
    fn unload_data(&mut self);
}

impl_trait_query!(Loadable);

#[cfg(feature = "asset_tracking")]
fn load_data(
    asset_server: Res<AssetServer>,
    mut assets_loading: ResMut<AssetsLoading>,
    mut query: Query<All<&mut dyn Loadable>, Added<IsLoaded>>,
) {
    for loadable_entity in query.iter_mut() {
        for mut loadable in loadable_entity {
            if let Err(e) = loadable.load_data(&asset_server, &mut assets_loading) {
                eprintln!("{}", e);
            }
        }
    }
}

#[cfg(not(feature = "asset_tracking"))]
fn load_data(
    asset_server: Res<AssetServer>,
    mut query: Query<All<&mut dyn Loadable>, Added<IsLoaded>>,
) {
    for loadable_entity in query.iter_mut() {
        for mut loadable in loadable_entity {
            if let Err(e) = loadable.load_data(&asset_server) {
                eprintln!("{}", e);
            }
        }
    }
}

fn unload_data(
    removed: RemovedComponents<IsLoaded>,
    mut query: Query<All<&mut dyn Loadable>>,
) {
    for entity in removed.iter() {
        if let Ok(loadable_comps) = query.get_mut(entity) {
            for mut loadable in loadable_comps {
                loadable.unload_data();
            }
        }
    }
}

/* pub fn load_data_system<L: Loadable>(
    asset_server: Res<AssetServer>,
    mut assets_loading: ResMut<AssetsLoading>,
    mut query: Query<&mut L, (With<IsLoaded>, Or<(Added<IsLoaded>, Added<L>)>)>,
) {
    for mut load_data in query.iter_mut() {
        if let Err(e) = load_data.load_data(&asset_server, &mut assets_loading) {
            println!("{}", e)
        }
    }
} */

/* pub fn unload_data_system<L: Loadable>(
    removed: RemovedComponents<IsLoaded>,
    mut query: Query<&mut L>,
) {
    for entity in removed.iter() {
        if let Ok(mut load_data) = query.get_mut(entity) {
            load_data.unload_data();
        }
    }
} */

pub trait LoadableEx: Component + Clone {
    fn load_data(&mut self, world: &mut World) -> anyhow::Result<()>;
    fn unload_data(&mut self, world: &mut World);
}

struct LoadCommand<T: LoadableEx> {
    loadables: HashSet<Entity>,
    phantom_data: PhantomData<T>,
}

impl<T: LoadableEx> Command for LoadCommand<T> {
    fn write(self, world: &mut World) {
        for &entity in self.loadables.iter() {
            let mut component = world.get_mut::<T>(entity).unwrap().clone();
            if let Err(e) = component.load_data(world) {
                println!("{}", e)
            }
            world.entity_mut(entity).insert(component);
        }
    }
}

struct UnloadCommand<T: LoadableEx> {
    unloadables: HashSet<Entity>,
    phantom_data: PhantomData<T>,
}

impl<T: LoadableEx> Command for UnloadCommand<T> {
    fn write(self, world: &mut World) {
        for &entity in self.unloadables.iter() {
            let mut component = world.get_mut::<T>(entity).unwrap().clone();
            component.unload_data(world);
            world.entity_mut(entity).insert(component);
        }
    }
}

pub fn load_ex_data_system<L: LoadableEx>(
    mut commands: Commands,
    query: Query<Entity, (With<L>, With<IsLoaded>, Or<(Added<IsLoaded>, Added<L>)>)>,
) {
    let load_list: HashSet<Entity> = query.iter().collect();

    if !load_list.is_empty() {
        commands.add(LoadCommand::<L> {
            loadables: load_list,
            phantom_data: PhantomData,
        })
    }
}

pub fn unload_ex_data_system<L: LoadableEx>(
    mut commands: Commands,
    removed: RemovedComponents<IsLoaded>,
    query: Query<Entity, With<L>>,
) {
    let removed_list: HashSet<Entity> = removed.iter().collect();
    let unloadable_list: HashSet<Entity> = query.iter().collect();

    let intersection: HashSet<Entity> = removed_list
        .intersection(&unloadable_list)
        .cloned()
        .collect();

    if !intersection.is_empty() {
        commands.add(UnloadCommand::<L> {
            unloadables: intersection,
            phantom_data: PhantomData,
        })
    }
}

pub trait AppRegisterLoadExt {
    fn register_loadable<L: Loadable + Component>(&mut self) -> &mut Self;
    fn register_loadable_ex<L: LoadableEx>(&mut self) -> &mut Self;
}

impl AppRegisterLoadExt for App {
    fn register_loadable<L: Loadable + Component>(&mut self) -> &mut Self {
        self.register_component_as::<dyn Loadable, L>();
/*         self.add_system_to_stage(CoreStage::PostUpdate, load_data_system::<L>)
            .add_system_to_stage(CoreStage::PostUpdate, unload_data_system::<L>); */

        self
    }

    fn register_loadable_ex<L: LoadableEx>(&mut self) -> &mut Self {
        self.add_system_to_stage(CoreStage::PostUpdate, load_ex_data_system::<L>)
            .add_system_to_stage(CoreStage::PostUpdate, unload_ex_data_system::<L>);

        self
    }
}
