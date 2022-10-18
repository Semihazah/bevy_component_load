use anyhow::{Ok, anyhow};
use bevy::{prelude::*, asset::AssetPlugin};

use crate::{BevyComponentLoadPlugin, Loadable, AppRegisterLoadExt, IsLoaded};

fn setup_app(app: &mut App) {
    app
    .add_plugins(MinimalPlugins)
    .add_plugin(AssetPlugin)
    .add_plugin(BevyComponentLoadPlugin)
    ;

    #[cfg(feature = "asset_tracking")]
    app
    .add_plugin(iyes_progress::ProgressPlugin::new(LoadingState::Loading).continue_to(LoadingState::NotLoading).track_assets())
    .add_state(LoadingState::Loading)
    ;
}

#[test]
fn test_loading() -> anyhow::Result<()>{
    let mut app = App::new();
    setup_app(&mut app);
    app
    .register_loadable::<Foo>();
    let foo_entity = app.world
    .spawn()
    .insert(Foo::default())
    .insert(IsLoaded)
    .id();

    app.update();

    let foo = app.world.get::<Foo>(foo_entity).ok_or(anyhow!("Unable to find Foo on entity!"))?;
    assert!(foo.message == Some("Hello World!".to_string()));

    app.world.entity_mut(foo_entity).remove::<IsLoaded>();
    
    app.update();

    let foo = app.world.get::<Foo>(foo_entity).ok_or(anyhow!("Unable to find Foo on entity!"))?;
    assert!(foo.message == None);

    Ok(())
}

#[derive(Component, Default)]
struct Foo {
    message: Option<String>,
}

#[cfg(not(feature = "asset_tracking"))]
impl Loadable for Foo {
    fn load_data(
            &mut self,
            _asset_server: &Res<AssetServer>,
            //_loading: &mut ResMut<iyes_progress::prelude::AssetsLoading>,
        ) -> anyhow::Result<()> {
        self.message = Some("Hello World!".to_string());
        Ok(())
    }

    fn unload_data(&mut self) {
        self.message = None;
    }
}

#[cfg(feature = "asset_tracking")]
impl Loadable for Foo {
    fn load_data(
            &mut self,
            _asset_server: &Res<AssetServer>,
            _loading: &mut ResMut<iyes_progress::prelude::AssetsLoading>,
        ) -> anyhow::Result<()> {
        self.message = Some("Hello World!".to_string());
        Ok(())
    }

    fn unload_data(&mut self) {
        self.message = None;
    }
}

#[cfg(feature = "asset_tracking")]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum LoadingState {
    NotLoading,
    Loading,
}