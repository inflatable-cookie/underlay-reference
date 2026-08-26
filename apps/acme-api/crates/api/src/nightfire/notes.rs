use underlay_media::nightfire::{
    NightfireBlockMediaHandler, NightfireBlockMediaHandlerMap, NightfireBlockMediaReference,
    NightfireBlockMediaRegistration, NightfireMediaVisitContext,
};
use underlay_media::{MediaId, MediaUsageRole};
use underlay_nightfire::{BlockDescriptor, BlockRegistration};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotesBlockCategory {
    Content,
}

pub struct NotesGalleryMediaHandler;

impl NightfireBlockMediaHandler for NotesGalleryMediaHandler {
    fn extract_media_references(
        &self,
        context: &NightfireMediaVisitContext<'_>,
    ) -> underlay_media::MediaResult<Vec<NightfireBlockMediaReference>> {
        let Some(pages) = context
            .resolve_relative_pointer("/pages")
            .and_then(|value| value.as_array())
        else {
            return Ok(Vec::new());
        };

        let mut references = Vec::new();

        for (index, page) in pages.iter().enumerate() {
            let Some(media_id) = page
                .get("imageId")
                .and_then(|value| value.as_str())
                .and_then(|value| Uuid::parse_str(value).ok())
                .map(MediaId::from_uuid)
            else {
                continue;
            };

            references.push(NightfireBlockMediaReference::new(
                media_id,
                MediaUsageRole::Embedded,
                format!("/pages/{index}/imageId"),
            ));
        }

        Ok(references)
    }
}

pub fn notes_gallery_media_registration() -> NightfireBlockMediaRegistration {
    NightfireBlockMediaRegistration::new("notes.gallery", NotesGalleryMediaHandler)
}

pub fn notes_gallery_block_registration(
) -> BlockRegistration<NotesBlockCategory, NightfireBlockMediaRegistration> {
    BlockRegistration::new(
        BlockDescriptor::new(
            "notes.gallery",
            "Image Gallery",
            NotesBlockCategory::Content,
        ),
        notes_gallery_media_registration(),
    )
}

pub fn notes_block_registrations(
) -> [BlockRegistration<NotesBlockCategory, NightfireBlockMediaRegistration>; 1] {
    [notes_gallery_block_registration()]
}

pub fn build_notes_media_registry() -> NightfireBlockMediaHandlerMap {
    NightfireBlockMediaHandlerMap::from_block_registrations(notes_block_registrations())
}
