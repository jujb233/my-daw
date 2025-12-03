## (i18n) Internationalization

This project includes a basic i18n helper at `src/i18n.ts`. By default the UI uses Chinese (`zh`).

- To switch locale at runtime (for dev/testing), import `setLocale` and call it, e.g.: `setLocale('en')` in your code.
- Add new translation keys to `src/i18n.ts` and use `t('<key>')` in components to render translated text.

Note: Only front-end strings are translated in this patch. Backend plugin names (e.g. `Mixer`) remain unchanged; if you want those localized as well, update the display names when the frontend renders them.
