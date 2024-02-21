macro_rules! impl_asset_loader {
    (
        $asset:ident
        $extensions:expr;
        $($field:ident)*
    ) => {
        paste::paste! {
            #[derive(Default)]
            struct [< $asset AssetLoader >];

            impl bevy::asset::AssetLoader for [< $asset AssetLoader >] {
                type Asset = $asset;

                type Settings = ();

                type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

                fn load<'a>(
                    &'a self,
                    reader: &'a mut bevy::asset::io::Reader,
                    _settings: &'a Self::Settings,
                    load_context: &'a mut bevy::asset::LoadContext,
                ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
                    use $crate::asset_ref::Loadable;
                    Box::pin(async move {
                        let mut bytes = Vec::new();
                        reader.read_to_end(&mut bytes).await?;
                        let mut loaded = ron::de::from_bytes::<$asset>(&bytes)?;

                        $(
                            loaded.$field.load(load_context);
                        )*

                        Ok(loaded)
                    })
                }

                fn extensions(&self) -> &[&str] {
                    $extensions
                }
            }
        }
    };
}
pub(crate) use impl_asset_loader;
