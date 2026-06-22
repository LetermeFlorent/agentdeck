// Adapter Claude Code : process persistant, entrée/sortie stream-json. Héberge aussi les
// helpers partagés (emit / write_images / ImageInput) réutilisés par les autres adapters.
pub mod claude_code;
pub mod claude_spawn;
pub mod claude_stream;
