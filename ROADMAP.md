# Roadmap
This document describes the current state of Iced and some of the most important next steps we should take before it can become a production-ready GUI library. This list keeps the short term new features in sight in order to coordinate work and discussion. Therefore, it is not meant to be exhaustive.

## Next steps
Most of the work related to these features needs to happen in the `iced_native` path of the ecosystem, as the web already supports many of them.

Once a step is completed, it is collapsed and added to this list:

  * [x] Scrollables / Clippables ([#24])
  * [x] Text input widget ([#25])
  * [x] TodoMVC example ([#26])
  * [x] Async actions ([#27])
  * [x] Custom layout engine ([#52])

[#24]: https://github.com/hecrj/iced/issues/24
[#25]: https://github.com/hecrj/iced/issues/25
[#26]: https://github.com/hecrj/iced/issues/26
[#28]: https://github.com/hecrj/iced/issues/28
[#52]: https://github.com/hecrj/iced/pull/52

### Multi-window support ([#27])
Open and control multiple windows at runtime.

I think this could be achieved by implementing an additional trait in `iced_winit` similar to `Application` but with a slightly different `view` method, allowing users to control what is shown in each window.

This approach should also allow us to perform custom optimizations for this particular use case.

[#27]: https://github.com/hecrj/iced/issues/27

### Event subscriptions ([#29])
Besides performing async actions on demand, most applications also need to listen to events passively. An example of this could be a WebSocket connection, where messages can come in at any time.

The idea here is to also follow [Elm]'s footsteps. We can add a method `subscriptions(&self) -> Subscription<Message>` to `Application` and keep the subscriptions alive in the runtime.

The challenge here is designing the public API of subscriptions. Ideally, users should be able to create their own subscriptions and the GUI runtime should keep them alive by performing _subscription diffing_ (i.e. detecting when a subscription is added, changed, or removed). For this, we can take a look at [Elm] for inspiration.

[#29]: https://github.com/hecrj/iced/issues/29

### Layers ([#30])
Currently, Iced assumes widgets cannot be laid out on top of each other. We should implement support for multiple layers of widgets.

This is a necessary feature to implement many kinds of interactables, like dropdown menus, select fields, etc.

`iced_native` will need to group widgets to perform layouting and process some events first for widgets positioned on top.

`iced_wgpu` will also need to process the scene graph and sort draw calls based on the different layers.

[#30]: https://github.com/hecrj/iced/issues/30

### Animations ([#31])
Allow widgets to request a redraw at a specific time.

This is a necessary feature to render loading spinners, a blinking text cursor, GIF images, etc.

[`winit`] allows flexible control of its event loop. We may be able to use [`ControlFlow::WaitUntil`](https://docs.rs/winit/0.20.0-alpha3/winit/event_loop/enum.ControlFlow.html#variant.WaitUntil) for this purpose.

[#31]: https://github.com/hecrj/iced/issues/31

### Canvas widget ([#32])
A widget to draw freely in 2D or 3D. It could be used to draw charts, implement a Paint clone, a CAD application, etc.

As a first approach, we could expose the underlying renderer directly here, and couple this widget with it ([`wgpu`] for now). Once [`wgpu`] gets WebGL or WebGPU support, this widget will be able to run on the web too. The renderer primitive could be a simple texture that the widget draws to.

In the long run, we could expose a renderer-agnostic abstraction to perform the drawing.

[#32]: https://github.com/hecrj/iced/issues/32

### Text shaping and font fallback ([#33])
[`wgpu_glyph`] uses [`glyph_brush`], which in turn uses [`rusttype`]. While the current implementation is able to layout text quite nicely, it does not perform any [text shaping].

[Text shaping] with font fallback is a necessary feature for any serious GUI toolkit. It unlocks support to truly localize your application, supporting many different scripts.

The only available library that does a great job at shaping is [HarfBuzz], which is implemented in C++. [`skribo`] seems to be a nice [HarfBuzz] wrapper for Rust.

This feature will probably imply rewriting [`wgpu_glyph`] entirely, as caching will be more complicated and the API will probably need to ask for more data.

[#33]: https://github.com/hecrj/iced/issues/33
[`rusttype`]: https://github.com/redox-os/rusttype
[text shaping]: https://en.wikipedia.org/wiki/Complex_text_layout
[HarfBuzz]: https://github.com/harfbuzz/harfbuzz
[`skribo`]: https://github.com/linebender/skribo

### Grid layout and text layout ([#34])
Currently, `iced_native` only supports flexbox items. For instance, it is not possible to create a grid of items or make text float around an image.

We will need to enhance the layouting engine to support different strategies and improve the way we measure text to lay it out in a more flexible way.

[#34]: https://github.com/hecrj/iced/issues/34

## Ideas that may be worth exploring

### Reuse existing 2D renderers
While I believe [`wgpu`] has a great future ahead of it, implementing `iced_wgpu` and making it performant will definitely be a challenge.

We should keep an eye on existing 2D graphic libraries, like [`piet`] or [`pathfinder`], and give them a try once/if they mature a bit more.

The good news here is that most of Iced is renderer-agnostic, so changing the rendering strategy, if we deem it worth it, should be really easy. Also, a 2D graphics library will expose a higher-level API than [`wgpu`], so implementing a new renderer on top of it should be fairly straightforward.

[`piet`]: https://github.com/linebender/piet
[`pathfinder`]: https://github.com/servo/pathfinder

### Remove explicit state handling and lifetimes
Currently, `iced_native` forces users to provide the local state of each widget. While this could be considered a really pure form of describing a GUI, it makes some optimizations harder because of the borrow checker.

The current borrow checker is not able to detect a drop was performed before reassigning a value to a mutable variable. Thus, keeping the generated widgets in `Application::view` alive between iterations of the event loop is not possible, which forces us to call this method quite often. `unsafe` could be used to workaround this, but it would feel fishy.

We could take a different approach, and keep some kind of state tree decoupled from the actual widget definitions. This would force us to perform diffing of nodes, as the widgets will represent the desired state and not the whole state.

Once the state lifetime of widgets is removed, we could keep them alive between iterations and even make `Application::view` take a non-mutable reference. This would also improve the end-user API, as it will not be necessary to constantly provide mutable state to widgets.

This is a big undertaking and introduces a new set of problems. We should research and consider the implications of this approach in detail before going for it.

### Improve style definitions
As of now, each widget defines its own styling options with methods, following the builder pattern.

A unified way of defining and reusing styles would be great. I think we must avoid replicating CSS, we should try to stay as type-safe, explicit, and intuitive as possible.

I think many different ideas in [`elm-ui`] could serve as an inspiration.

[`elm-ui`]: https://www.youtube.com/watch?v=Ie-gqwSHQr0

### Try a different font rasterizer
[`wgpu_glyph`] depends indirectly on [`rusttype`]. We may be able to gain performance by using a different font rasterizer. [`fontdue`], for instance, has reported noticeable speedups.

[`fontdue`]: https://github.com/mooman219/fontdue

### Connect `iced_web` with `web-view`
It may be interesting to try to connect `iced_web` with [`web-view`]. It would give users a feature-complete renderer for free, and applications would still be leaner than with Electron.

[`web-view`]: https://github.com/Boscop/web-view

### Implement a lazy widget
Once we remove state lifetimes from widgets, we should be able to implement a widget storing a function that generates additional widgets. The runtime would then be able to control when to call this function and cache the generated widgets while some given value does not change.

This could be very useful to build very performant user interfaces with a lot of different items.

[Elm does it very well!](https://guide.elm-lang.org/optimization/lazy.html)

[Elm]: https://elm-lang.org/
[`winit`]: https://github.com/rust-windowing/winit
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs
[`wgpu_glyph`]: https://github.com/hecrj/wgpu_glyph
[`glyph_brush`]: https://github.com/alexheretic/glyph-brush