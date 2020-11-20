use super::*;
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        // 2d camera
        .spawn(UiCameraComponents::default())
        // texture
        .spawn(TextComponents {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "".to_string(),
                font: asset_server.load("FiraSans-Bold.ttf"),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            },
            ..Default::default()
        });
}
