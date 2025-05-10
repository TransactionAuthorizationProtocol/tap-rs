---
trigger: glob
globs: tap-msg/*
---

When implementing any messages types here always understand the full specification as defined in @prds/taips/message.md. These specs should always be seen as the source of truth.

Always follow the rules in @tap-msg/src/tap_message_implementation_guide.md when implementing new message types.

Never change the main format of the messages to fix an implementation problem in another cask.