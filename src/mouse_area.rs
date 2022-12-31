use std::cell::RefCell;

use iced::{mouse::Button, touch, Element, Length};
use iced_native::{
    layout, mouse,
    widget::{tree, Tree},
    Layout, Widget,
};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct MouseState {
    pub hovered: bool,
    pub pressed: bool, // TODO left/middle/right, etc...
                       // TODO mouse position, relative vs absolute
}

pub struct MouseArea<'a, Message, Renderer> {
    view: Box<dyn Fn(MouseState) -> Element<'a, Message, Renderer> + 'a>,
    content: RefCell<Content<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    pub fn new<T>(view: impl Fn(MouseState) -> T + 'a) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            view: Box::new(move |mouse_state| view(mouse_state).into()),
            content: RefCell::new(Content {
                mouse_state: Default::default(),
                element: None,
            }),
        }
    }
}

impl<'a, Message, Renderer> From<MouseArea<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    Message: 'a,
{
    fn from(value: MouseArea<'a, Message, Renderer>) -> Self {
        Self::new(value)
    }
}

struct Content<'a, Message, Renderer> {
    mouse_state: MouseState,
    element: Option<Element<'a, Message, Renderer>>,
}

enum UpdateStatus {
    Unchanged,
    Updated,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer> {
    fn update(
        &mut self,
        mouse_state: MouseState,
        view: impl Fn(MouseState) -> Element<'a, Message, Renderer>,
    ) -> UpdateStatus {
        if mouse_state == self.mouse_state && self.element.is_some() {
            return UpdateStatus::Unchanged;
        }
        self.mouse_state = mouse_state;
        self.element = Some(view(mouse_state));
        UpdateStatus::Updated
    }

    fn resolve(
        &mut self,
        view: impl Fn(MouseState) -> Element<'a, Message, Renderer>,
    ) -> &mut Element<'a, Message, Renderer> {
        if self.element.is_none() {
            self.element = Some(view(Default::default()));
        }
        self.element.as_mut().unwrap()
    }
}

struct State {
    mouse_state: MouseState,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for MouseArea<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .borrow_mut() // TODO consider making this self.resolve()
            .resolve(&self.view)
            .as_widget()
            .draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State>();
        self.content
            .borrow_mut()
            .update(state.mouse_state, &self.view);
        tree.diff_children(&[self.content.borrow_mut().resolve(&self.view).as_widget()]);
    }

    fn width(&self) -> Length {
        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget()
            .width()
    }

    fn height(&self) -> Length {
        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget()
            .height()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            mouse_state: Default::default(),
        })
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget()
            .layout(renderer, limits)
    }

    fn operate(
        &self,
        _state: &mut Tree,
        _layout: iced_native::Layout<'_>,
        _operation: &mut dyn iced_native::widget::Operation<Message>,
    ) {
        todo!() // Should cache resolved content element and forward widget operation
    }

    // TODO:
    //fn overlay<'b>(
    //    &'b mut self,
    //    tree: &'b mut Tree,
    //    layout: iced_native::Layout<'_>,
    //    renderer: &Renderer,
    //) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
    //    self.content
    //        .borrow_mut()
    //        .resolve(&self.view)
    //        .as_widget_mut()
    //        .overlay(&mut tree.children[0], layout, renderer)
    //}

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            self.content.borrow_mut().resolve(&self.view).as_widget(),
        )]
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        shell: &mut iced_native::Shell<'_, Message>,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<State>();
        state.mouse_state.hovered = layout.bounds().contains(cursor_position);
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(Button::Left))
            | iced::Event::Touch(touch::Event::FingerPressed { .. }) => {
                // TODO: Is touch
                // considered mouse or
                // should I make it
                // separate? I.e. pointer
                // vs mouse vs touch
                if state.mouse_state.hovered {
                    state.mouse_state.pressed = true;
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(Button::Left))
            | iced::Event::Touch(touch::Event::FingerLost { .. }) => {
                state.mouse_state.pressed = false;
            }
            _ => {}
        }

        if let UpdateStatus::Updated = self
            .content
            .borrow_mut()
            .update(state.mouse_state, &self.view)
        {
            shell.invalidate_widgets();
        }

        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget_mut()
            .on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget()
            .mouse_interaction(
                &tree.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
    }
}
