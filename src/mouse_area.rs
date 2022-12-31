use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    ops::Deref,
};

use iced::{mouse::Button, touch, Element, Length, Point, Rectangle, Size};
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
    //content: RefCell<Option<Element<'a, Message, Renderer>>>,
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    pub fn new(view: impl Fn(MouseState) -> Element<'a, Message, Renderer> + 'a) -> Self {
        //println!("MouseArea::new");
        Self {
            view: Box::new(view),
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

impl<'a, Message, Renderer> Content<'a, Message, Renderer> {
    fn update(
        &mut self,
        mouse_state: MouseState,
        view: impl Fn(MouseState) -> Element<'a, Message, Renderer>,
    ) {
        if mouse_state == self.mouse_state && self.element.is_some() {
            return;
        }
        self.mouse_state = mouse_state;
        self.element = Some(view(mouse_state));
    }

    fn resolve(
        &mut self,
        view: impl Fn(MouseState) -> Element<'a, Message, Renderer>,
    ) -> &mut Element<'a, Message, Renderer> {
        if self.element.is_none() {
            //println!("resolve: Warning: mouse area content approximated as we don't have mouse state yet");
            self.element = Some(view(Default::default()));
        } else {
            //println!("resolve: Using existing content");
        }
        self.element.as_mut().unwrap()
        //RefMut::map(self.content.borrow_mut(), |optional_content| {
        //    optional_content.as_mut().unwrap()
        //})
    }
}

//impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
//    fn resolve(&self, tree: &Tree) -> Element<'a, Message, Renderer> {
//        // TODO: Cache this so it can be reused in the current frame.
//        let state = tree.state.downcast_ref::<State>();
//        (self.view)(state.mouse_state.get())
//    }
//}

struct State {
    mouse_state: MouseState,
    //content_bounds: Cell<Rectangle>,
    //content_tree: RefCell<Tree>,
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
        //let content_layout = layout.children().next().unwrap();
        let content_layout = layout; // TODO

        self.content
            .borrow_mut() // TODO consider making this self.resolve()
            .resolve(&self.view)
            .as_widget()
            .draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_layout,
                cursor_position,
                viewport,
            );
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State>();
        // Construct the new
        //println!("diff: mouse state: {}", state.mouse_state.get().hovered);
        self.content
            .borrow_mut()
            .update(state.mouse_state, &self.view);
        tree.diff_children(&[self.content.borrow_mut().resolve(&self.view).as_widget()]);
    }

    fn width(&self) -> Length {
        // TODO: This may become out of date when the mouse state changes.
        // How do we let Iced know that the layout is invalidated?
        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget()
            .width()
    }

    fn height(&self) -> Length {
        // TODO: This may become out of date when the mouse state changes.
        // How do we let Iced know that the layout is invalidated?
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
        //layout::Node::with_children(
        //    limits.max(),
        //    vec![self.resolve().as_widget().layout(renderer, limits)],
        //)
    }

    fn operate(
        &self,
        _state: &mut Tree,
        _layout: iced_native::Layout<'_>,
        _operation: &mut dyn iced_native::widget::Operation<Message>,
    ) {
        todo!() // Should cache resolved content element and forward widget operation
    }

    //fn overlay<'a>(
    //        &'a mut self,
    //        _state: &'a mut Tree,
    //        _layout: iced_native::Layout<'_>,
    //        _renderer: &Renderer,
    //    ) -> Option<iced_native::overlay::Element<'a, Message, Renderer>> {
    //
    //}

    fn children(&self) -> Vec<Tree> {
        // TODO: Hmmmmmmmmmm.... is this borrowing for too long??
        //vec![Tree::new(self.content.borrow().as_ref().expect(
        //    "widget::diff should be called before asking for children",
        //))]
        // TODO: above currently hits assert.
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
        //todo!() // Should cache resolved content elmeent and forward event
        //iced::event::Status::Ignored
        //let content_layout = layout.children().next().unwrap();
        let content_layout = layout; // TODO

        let state = tree.state.downcast_mut::<State>();
        state.mouse_state.hovered = content_layout.bounds().contains(cursor_position);
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
        self.content
            .borrow_mut()
            .update(state.mouse_state, &self.view);

        self.content
            .borrow_mut()
            .resolve(&self.view)
            .as_widget_mut()
            .on_event(
                &mut tree.children[0],
                event,
                content_layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )

        //match event {
        //    iced::Event::Mouse(mouse::Event::)
        //    _ => event::Status::Ignored,
        //}
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: iced_native::Layout<'_>,
        _cursor_position: iced::Point,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        // TODO: should cache resolved content element and forward event
        mouse::Interaction::Idle
    }
}

// struct MouseArea<'a, Message, Renderer> {
//     content: Element<'a, Message, Renderer>,
// }
//
// enum Content<'a, Message, Renderer> {
//     Oblivious(Element<'a, Message, Renderer>),
//     Interested(Box<dyn Fn(MouseState) -> Element<'a, Message, Renderer>>),
// }
//
// impl<'a, Message, Renderer> From<Element<'a, Message, Renderer>>
//     for Content<'a, Message, Renderer>
// {
// }
//
// impl<'a, Message, Renderer> From<Box<dyn Fn(MouseState) -> Element<'a, Message, Renderer>>>
//     for Content<'a, Message, Renderer>
// {
// }
//
// fn mouse_area<'a, Message, Renderer>(
//     content: impl Into<Content<'a, Message, Renderer>>,
// ) -> MouseArea<'a, Message, Renderer> {
// }
//
//
// struct Inspectable<T> {
//     id: widget::Id,
//     _type: PhantomData<T>,
// }
//
// type MouseAreaId = Inspectable<MouseState>;
//
// struct Inspect<'a, Message, Renderer, T> {
//     target: Inspectable<T>,
//     view: Box<dyn Fn(T) -> Element<'a, Message, Renderer>>,
// }
//
// fn x() {
//     let item_mouse_area = MouseAreaId::new();
//     row![
//         mouse_area(button("hello")).id(item_mouse_area),
//         inspect(item_mouse_area, |mouse_state| {
//             if mouse_state.is_hovered() {
//                 "world"
//             } else {
//                 ""
//             }
//         }),
//     ]
// }
