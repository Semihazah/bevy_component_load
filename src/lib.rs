use bevy::{
    ecs::{component::Component},
    prelude::*,
    reflect::FromReflect,
};
use bevy_trait_query::{All, RegisterExt};

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

#[bevy_trait_query::queryable]
pub trait Loadable: 'static {
    fn load_data(
        &mut self,
        asset_server: &Res<AssetServer>,

        #[cfg(feature = "asset_tracking")]
        loading: &mut ResMut<AssetsLoading>,
    ) -> anyhow::Result<()>;
    fn unload_data(&mut self);
}

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

pub trait AppRegisterLoadExt {
    fn register_loadable<L: Loadable + Component>(&mut self) -> &mut Self;
}

impl AppRegisterLoadExt for App {
    fn register_loadable<L: Loadable + Component>(&mut self) -> &mut Self {
        self.register_component_as::<dyn Loadable, L>();
/*         self.add_system_to_stage(CoreStage::PostUpdate, load_data_system::<L>)
            .add_system_to_stage(CoreStage::PostUpdate, unload_data_system::<L>); */

        self
    }
}
