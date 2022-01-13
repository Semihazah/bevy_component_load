use std::marker::PhantomData;

use bevy::{
    ecs::{component::Component, system::Command},
    prelude::*,
    reflect::FromReflect,
};
use bevy_loading::prelude::AssetsLoading;

pub struct BevyComponentLoadPlugin;

impl Plugin for BevyComponentLoadPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsLoaded>();
    }
}

#[derive(Reflect, FromReflect, Default, Debug, Component)]
#[reflect(Component)]
pub struct IsLoaded;

pub trait Loadable: Component {
    fn load_data(
        &mut self,
        asset_server: &Res<AssetServer>,
        loading: &mut ResMut<AssetsLoading>,
    ) -> anyhow::Result<()>;
    fn unload_data(&mut self);
}

pub fn load_data_system<L: Loadable>(
    asset_server: Res<AssetServer>,
    mut assets_loading: ResMut<AssetsLoading>,
    mut query: Query<&mut L, (With<IsLoaded>, Or<(Added<IsLoaded>, Added<L>)>)>,
) {
    for mut load_data in query.iter_mut() {
        if let Err(e) = load_data.load_data(&asset_server, &mut assets_loading) {
            println!("{}", e)
        }
    }
}

pub fn unload_data_system<L: Loadable>(
    removed: RemovedComponents<IsLoaded>,
    mut query: Query<&mut L>,
) {
    for entity in removed.iter() {
        if let Ok(mut load_data) = query.get_mut(entity) {
            load_data.unload_data();
        }
    }
}

pub trait LoadableEx: Component + Clone {
    fn load_data(&mut self, world: &mut World) -> anyhow::Result<()>;
    fn unload_data(&mut self);
}

struct LoadCommand<T: LoadableEx> {
    loadables: Vec<Entity>,
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

pub fn load_ex_data_system<L: LoadableEx>(
    mut commands: Commands,
    query: Query<Entity, (With<L>, With<IsLoaded>, Or<(Added<IsLoaded>, Added<L>)>)>,
) {
    let load_list: Vec<Entity> = query.iter().collect();

    if !load_list.is_empty() {
        commands.add(LoadCommand::<L> {
            loadables: load_list,
            phantom_data: PhantomData,
        })
    }
}

pub fn unload_ex_data_system<L: LoadableEx>(
    removed: RemovedComponents<IsLoaded>,
    mut query: Query<&mut L>,
) {
    for entity in removed.iter() {
        if let Ok(mut load_data) = query.get_mut(entity) {
            load_data.unload_data();
        }
    }
}

pub trait AppRegisterLoadExt {
    fn register_loadable<L: Loadable>(&mut self) -> &mut Self;
    fn register_loadable_ex<L: LoadableEx>(&mut self) -> &mut Self;
}

impl AppRegisterLoadExt for App {
    fn register_loadable<L: Loadable>(&mut self) -> &mut Self {
        self.add_system_to_stage(CoreStage::PostUpdate, load_data_system::<L>.system())
            .add_system_to_stage(CoreStage::PostUpdate, unload_data_system::<L>.system());

        self
    }

    fn register_loadable_ex<L: LoadableEx>(&mut self) -> &mut Self {
        self.add_system_to_stage(CoreStage::PostUpdate, load_ex_data_system::<L>.system())
            .add_system_to_stage(CoreStage::PostUpdate, unload_ex_data_system::<L>.system());

        self
    }
}
