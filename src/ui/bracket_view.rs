use std::collections::HashMap;

use log::{error};
use wasm_bindgen::{JsCast};
use web_sys::{ResizeObserver, HtmlElement, HtmlDivElement, MouseEvent, HtmlButtonElement, DomRect, window, HtmlTemplateElement, Element, HtmlCanvasElement, CanvasRenderingContext2d};

use crate::{dom::{create_element}, model::tournament::{StageId, TournamentId, StageKind, FixtureId, FixtureTeam, Outcome, FixtureInput}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

use super::create_callback_with_arg;

pub struct BracketView {
    id: UiElementId,
    tournament_id: Option<TournamentId>,
    stage_id: Option<StageId>,
    linked_outline_id: UiElementId,

    dom_root: HtmlElement,
    canvas_container: HtmlDivElement, // <canvas> can't contain other elements, so we put our fixture divs alongside the canvas in this container
    canvas: HtmlCanvasElement,
    canvas_context: CanvasRenderingContext2d,
    fixture_divs: HashMap<FixtureId, HtmlDivElement>,

    #[allow(dyn_drop)]
    closures: Vec<Box<dyn Drop>>,

    current_drag: Option<DragInfo>,
    current_connecting_line: Option<ConnectingLine>,
}

struct ConnectingLine {
    start_fixture_id: FixtureId,
    start_outcome: Outcome,
    dragging_end: (i32, i32),
    end_fixture_input: Option<(FixtureId, FixtureInput)>
}

struct DragInfo {
    fixture_id: FixtureId,
    start_offset: (f64, f64),
}

impl BracketView {
    pub fn get_id(&self) -> UiElementId {
        self.id
    }

    pub fn tournament_changed(&mut self, model: &Model, tournament_id: TournamentId) {
        if Some(tournament_id) == self.tournament_id {
            self.refresh(model);
        }
    }

    pub fn process_events(&mut self, events: &EventList, model: &Model) {
        for e in events.get_events() {
            match e {
                Event::SelectedTournamentAndStageChanged { source, new_tournament_id, new_stage_id } if *source == self.linked_outline_id => {
                    self.tournament_id = *new_tournament_id;
                    self.stage_id = *new_stage_id;
                    self.refresh(model);
                }
                _ => (),
            }
        }
    }

    pub fn get_dom_root(&self) -> &HtmlElement {
        &self.dom_root
    }

    pub fn new(id: UiElementId, model: &Model, linked_outline_id: UiElementId) -> BracketView {
        let dom_root = create_element::<HtmlElement>("div");
        dom_root.set_inner_html("<h3>Bracket</h3>");

        let canvas_container = create_element::<HtmlDivElement>("div");
        canvas_container.set_class_name("bracket-view-canvas-container");
        canvas_container.style().set_property("position", "relative").expect("Failed to set property"); // For children to be absolutely positioned relative to this.
        dom_root.append_child(&canvas_container).expect("Failed to append child");

        #[allow(dyn_drop)]
        let mut closures: Vec<Box<dyn Drop>> = vec![];

        let canvas = create_element::<HtmlCanvasElement>("canvas");
        canvas.set_class_name("bracket-view-canvas");

        let canvas_resize_closure = Box::new(create_callback(move |model, ui| {
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                this.on_canvas_resize(model);
            }
        }));
        let resize_observer = ResizeObserver::new(canvas_resize_closure.as_ref().as_ref().unchecked_ref()).expect("Failedto create ResizeObserver");
        resize_observer.observe(&canvas);
        closures.push(canvas_resize_closure); // Needs to be kept alive


        canvas_container.append_child(&canvas).expect("Failed to append child");
        let canvas_context: CanvasRenderingContext2d = canvas.get_context("2d").expect("Failed to get context").expect("Failed to get context").dyn_into().expect("Failed cast");

        let dblclick_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                this.on_background_dblclick(model, e);
            }
        }));
        canvas_container.set_ondblclick(Some(dblclick_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(dblclick_closure); // Needs to be kept alive

        let mousemove_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                this.on_background_mousemove(model, e);
            }
        }));
        canvas_container.set_onmousemove(Some(mousemove_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(mousemove_closure); // Needs to be kept alive

        let mouseup_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                this.on_background_mouseup(model, e);
            }
        }));
        canvas_container.set_onmouseup(Some(mouseup_closure.as_ref().as_ref().unchecked_ref()));
        closures.push(mouseup_closure); // Needs to be kept alive


        let mut result = BracketView { id, tournament_id: None, stage_id: None, linked_outline_id, dom_root, canvas_container, canvas, canvas_context,
            fixture_divs: HashMap::<FixtureId, HtmlDivElement>::new(), closures,
            current_drag: None, current_connecting_line: None };

        result.refresh(model);

        result
    }

    fn refresh(&mut self, model: &Model) {
        let mut show = false;
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(s) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { .. } = s.kind {
                    show = true;
                }
            }
        }
        self.dom_root.style().set_property("display",
            if show { "block" } else { "none" }).expect("Failed to set style");

        if let Ok(fixture_divs) = self.canvas_container.query_selector_all(".fixture") {
            for i in 0..fixture_divs.length() {
                fixture_divs.item(i).map(|f| f.dyn_into::<Element>().unwrap().remove());
            }
        }
        //TODO: delete closures?

        self.fixture_divs.clear();

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { fixtures } = &stage.kind {
                    let template: HtmlTemplateElement = window().unwrap().document().unwrap().get_element_by_id("bracket-view-fixture-template").expect("Failed to find element")
                        .dyn_into().expect("Cast failed");

                    for (fid, f) in fixtures {
                        let new_div: HtmlDivElement = template.content().first_element_child().expect("Missing element").clone_node_with_deep(true).expect("Clone failed").dyn_into().expect("Cast failed");
                        new_div.set_class_name("fixture");

                        // Note that the cloned element needs to be added to the document before doing other stuff with it, otherwise things don't seem to work
                        self.canvas_container.append_child(&new_div).expect("Failed to append child");


                        // let new_div: HtmlDivElement = create_element::<HtmlDivElement>("div");
                        new_div.query_selector("span[name=fixture-id]").expect("Missing entry").expect("Missing entry").set_text_content(Some(&fid.to_string()));

                        let team_a_span = new_div.query_selector("span[name=team-a]").expect("Missing entry").expect("Missing entry");
                        team_a_span.set_text_content(Some(&format!("{:?}", f.team_a.to_pretty_desc(stage))));

                        let team_b_span = new_div.query_selector("span[name=team-b]").expect("Missing entry").expect("Missing entry");
                        team_b_span.set_text_content(Some(&format!("{:?}", f.team_b.to_pretty_desc(stage))));

                        new_div.style().set_property("position", "absolute").expect("Failed to set property");
                        new_div.style().set_property("left", &f.layout.0.to_string()).expect("Failed to set property");
                        new_div.style().set_property("top", &f.layout.1.to_string()).expect("Failed to set property");

                        let delete_button: HtmlButtonElement = new_div.query_selector("button[name=delete-button]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let id = self.id;
                        let fid = *fid;
                        let click_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
                                this.on_delete_fixture_button_click(model, fid);
                            }
                        }));
                        delete_button.set_onclick(Some(click_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(click_closure); // Needs to be kept alive

                        let drag_handle: HtmlElement = new_div.query_selector("span[name=drag-handle]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let mousedown_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_drag_handle_mousedown(model, fid, e);
                            }
                        }));
                        drag_handle.set_onmousedown(Some(mousedown_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mousedown_closure); // Needs to be kept alive

                        let team_a_handle: HtmlElement = new_div.query_selector("span[name=team-a]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let mouseenter_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_input_mouseenter(model, fid, FixtureInput::TeamA);
                            }
                        }));
                        team_a_handle.set_onmouseenter(Some(mouseenter_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mouseenter_closure); // Needs to be kept alive
                        let mouseleave_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_input_mouseleave(model, fid, FixtureInput::TeamA);
                            }
                        }));
                        team_a_handle.set_onmouseleave(Some(mouseleave_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mouseleave_closure); // Needs to be kept alive

                        let team_b_handle: HtmlElement = new_div.query_selector("span[name=team-b]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let mouseenter_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_input_mouseenter(model, fid, FixtureInput::TeamB);
                            }
                        }));
                        team_b_handle.set_onmouseenter(Some(mouseenter_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mouseenter_closure); // Needs to be kept alive
                        let mouseleave_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_input_mouseleave(model, fid, FixtureInput::TeamB);
                            }
                        }));
                        team_b_handle.set_onmouseleave(Some(mouseleave_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mouseleave_closure); // Needs to be kept alive

                        let winner_handle: HtmlElement = new_div.query_selector("td[name=winner-handle]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let mousedown_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_outcome_handle_mousedown(model, fid, Outcome::Winner);
                            }
                        }));
                        winner_handle.set_onmousedown(Some(mousedown_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mousedown_closure); // Needs to be kept alive

                        let loser_handle: HtmlElement = new_div.query_selector("td[name=loser-handle]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let mousedown_closure = Box::new(create_callback(move |model, ui| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_outcome_handle_mousedown(model, fid, Outcome::Loser);
                            }
                        }));
                        loser_handle.set_onmousedown(Some(mousedown_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(mousedown_closure); // Needs to be kept alive


                        self.fixture_divs.insert(fid, new_div);
                    }
                }
            }
        }

        self.redraw_canvas(model);
    }

    fn redraw_canvas(&self, model: &Model) {
        self.canvas_context.clear_rect(0 as f64, 0 as f64, self.canvas.width() as f64, self.canvas.height() as f64);

        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(stage) = model.get_stage(tournament_id, stage_id) {
                if let StageKind::Bracket { fixtures } = &stage.kind {
                    for (fid, f) in fixtures {
                        let get_start = |ft: &FixtureTeam| {
                            match ft {
                                FixtureTeam::Linked { fixture_id: linked_fixture_id, outcome } => self.get_fixture_outcome_handle_rect(*linked_fixture_id, *outcome),
                                _ => None
                            }
                        };

                        if let Some(start) = get_start(&f.team_a) {
                            let end = self.get_fixture_input_handle_rect(*fid, FixtureInput::TeamA).expect("This fixture must exist as it's the one we're looking at");
                            self.canvas_context.begin_path();
                            self.canvas_context.move_to(start.right() as f64, start.top() + start.height() / 2 as f64);
                            self.canvas_context.line_to(end.left() as f64, end.top() + end.height() / 2 as f64);
                            self.canvas_context.stroke();
                        }

                        if let Some(start) = get_start(&f.team_b) {
                            let end = self.get_fixture_input_handle_rect(*fid, FixtureInput::TeamB).expect("This fixture must exist as it's the one we're looking at");
                            self.canvas_context.begin_path();
                            self.canvas_context.move_to(start.right() as f64, start.top() + start.height() / 2 as f64);
                            self.canvas_context.line_to(end.left() as f64, end.top() + end.height() / 2 as f64);
                            self.canvas_context.stroke();
                        }
                    }
                }

                if let Some(connecting_line) = &self.current_connecting_line {
                    let start = self.get_fixture_outcome_handle_rect(connecting_line.start_fixture_id, connecting_line.start_outcome);
                    let end = match connecting_line.end_fixture_input {
                        None => connecting_line.dragging_end,
                        Some((end_fixture_id, end_fixture_input)) => {
                            let r = self.get_fixture_input_handle_rect(end_fixture_id, end_fixture_input);
                            match r {
                                Some(r) => (r.left() as i32, (r.top() + r.height() * 0.5) as i32),
                                None => connecting_line.dragging_end,
                            }
                        }
                    };

                    if let Some(start) = start {
                        self.canvas_context.begin_path();
                        self.canvas_context.move_to(start.right() as f64, start.top() + start.height() / 2 as f64);
                        self.canvas_context.line_to(end.0 as f64, end.1 as f64);
                        self.canvas_context.stroke();
                    }
                }
            }
        }
    }

    fn get_fixture_element_rect(&self, fixture_id: FixtureId, element_name: &str) -> Option<DomRect> {
        let canvas_rect = self.canvas_container.get_bounding_client_rect();
        if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
            let element = fixture_div.query_selector(&format!("[name={element_name}]")).expect("Missing entry").expect("Missing entry");
            let client_rect = element.get_bounding_client_rect();
            Some(DomRect::new_with_x_and_y_and_width_and_height(client_rect.x() - canvas_rect.x(),
                client_rect.y() - canvas_rect.y(), client_rect.width(), client_rect.height()).expect("Failed to make DomRect"))
        } else {
            None
        }
    }

    fn get_fixture_input_handle_rect(&self, fixture_id: FixtureId, input: FixtureInput) -> Option<DomRect> {
        let element_name = match input {
            FixtureInput::TeamA => "team-a",
            FixtureInput::TeamB => "team-b",
        };
        self.get_fixture_element_rect(fixture_id, element_name)
    }
    fn get_fixture_outcome_handle_rect(&self, fixture_id: FixtureId, outcome: Outcome) -> Option<DomRect> {
        let element_name = match outcome {
            Outcome::Winner => "winner-handle",
            Outcome::Loser => "loser-handle",
        };
        self.get_fixture_element_rect(fixture_id, element_name)
    }

    fn on_canvas_resize(&self, model: &Model) {
        self.canvas.set_width(self.canvas.client_width() as u32);
        self.canvas.set_height(self.canvas.client_height() as u32);

        self.redraw_canvas(model);
    }

    fn on_background_dblclick(&self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {

            let team_a = {
                // Add a new team to be used as the fixed input for this fixture
                let team_id = match model.add_team(tournament_id, stage_id, "ABC".to_string()) {
                    Some(t) => t,
                    None => {
                        error!("Failed to add team");
                        return;
                    }
                };
                FixtureTeam::Fixed(team_id)
            };

            let team_b = {
                // Add a new team to be used as the fixed input for this fixture
                let team_id = match model.add_team(tournament_id, stage_id, "ABC".to_string()) {
                    Some(t) => t,
                    None => {
                        error!("Failed to add team");
                        return;
                    }
                };
                FixtureTeam::Fixed(team_id)
            };

            if let None = model.add_fixture(tournament_id, stage_id, (e.offset_x(), e.offset_y()), team_a, team_b) {
                error!("Failed to add fixture");
            }
        }
    }

    fn on_delete_fixture_button_click(&self, model: &mut Model, fixture_id: FixtureId) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if window().unwrap().confirm_with_message(&format!("Are you sure you want to delete this fixture?")) == Ok(true) {
                //TODO: also delete the associated match if any??
                //TODO: delete any teams as fixed inputs?
                if let Err(_) = model.delete_fixture(tournament_id, stage_id, fixture_id) {
                    error!("Failed to delete fixture");
                }
            }
        }
    }

    fn on_fixture_drag_handle_mousedown(&mut self, _model: &mut Model, fixture_id: FixtureId, e: MouseEvent) {
        if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
            let fixture_div_rect = fixture_div.get_bounding_client_rect();
            self.current_drag = Some(DragInfo { fixture_id, start_offset: (e.client_x() as f64 - fixture_div_rect.left(), e.client_y() as f64 - fixture_div_rect.top()) });
        }
    }

    fn on_fixture_outcome_handle_mousedown(&mut self, _model: &mut Model, fixture_id: FixtureId, outcome: Outcome) {
        self.current_connecting_line = Some(ConnectingLine { start_fixture_id: fixture_id, start_outcome: outcome, dragging_end: (0, 0), end_fixture_input: None });
    }

    fn on_fixture_input_mouseenter(&mut self, _model: &mut Model, fixture_id: FixtureId, input: FixtureInput) {
        if let Some(connecting_line) = self.current_connecting_line.as_mut() {
            connecting_line.end_fixture_input = Some((fixture_id, input));
        }
    }

    fn on_fixture_input_mouseleave(&mut self, _model: &mut Model, _fixture_id: FixtureId, _input: FixtureInput) {
        if let Some(connecting_line) = self.current_connecting_line.as_mut() {
            connecting_line.end_fixture_input = None;
        }
    }

    fn on_background_mousemove(&mut self, model: &mut Model, e: MouseEvent) {
            // Get the cursor position relative to the bracket view
        // This won't be the same as e.offsetX/Y if we are currently dragging over a child element of the bracket view,
        // as we receive the mousemove event via bubbling, so e.target will be the child element, and e.offsetX/Y will be relative to that instead.
        let canvas_rect = self.canvas_container.get_bounding_client_rect();
        let x = e.client_x() as f64 - canvas_rect.left();
        let y = e.client_y() as f64 - canvas_rect.top();

         if let Some(drag_info) = &self.current_drag {
            if let Some(fixture_div) = self.fixture_divs.get(&drag_info.fixture_id) {
                // debug!("target rect = {},{}. root rect = {},{}. updated to {x} {y}", target_rect.left(), target_rect.top(), canvas_rect.left(), canvas_rect.top());
                let x = x - drag_info.start_offset.0;
                let y = y - drag_info.start_offset.1;

                fixture_div.style().set_property("left", &x.to_string()).expect("Failed to set property");
                fixture_div.style().set_property("top", &y.to_string()).expect("Failed to set property");

                self.redraw_canvas(model);
            }
        }

        if let Some(connecting_line) = self.current_connecting_line.as_mut() {
            connecting_line.dragging_end = (x as i32, y as i32);

            self.redraw_canvas(model);
        }
    }

    fn on_background_mouseup(&mut self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(drag_info) = &self.current_drag {
                let fixture_id = drag_info.fixture_id;
                let canvas_rect = self.canvas_container.get_bounding_client_rect();
                let x = e.client_x() as f64 - canvas_rect.left() - drag_info.start_offset.0;
                let y = e.client_y() as f64 - canvas_rect.top() - drag_info.start_offset.1;

                self.current_drag = None;
                if let Err(_) = model.set_fixture_layout(tournament_id, stage_id, fixture_id, (x as i32, y as i32)) {
                    error!("Failed to update fixture");
                }
            }

            if let Some(connecting_line) = self.current_connecting_line.as_mut() {
                if let Some((end_fixture_id, end_fixture_input)) = connecting_line.end_fixture_input {
                    if let Err(_) = model.set_fixture_input(tournament_id, stage_id, end_fixture_id, end_fixture_input,
                        FixtureTeam::Linked { fixture_id: connecting_line.start_fixture_id, outcome: connecting_line.start_outcome })
                    {
                        error!("Failed to update fixture");
                    }
                }

                self.current_connecting_line = None;

                self.redraw_canvas(model);
            }
        }
    }
}
