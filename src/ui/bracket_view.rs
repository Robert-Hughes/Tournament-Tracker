use std::collections::HashMap;

use log::{error, debug};
use wasm_bindgen::{JsCast};
use web_sys::{ResizeObserver, HtmlElement, HtmlDivElement, MouseEvent, HtmlButtonElement, DomRect, window, HtmlSelectElement, HtmlOptionElement, HtmlTemplateElement, Element, HtmlCanvasElement, CanvasRenderingContext2d};

use crate::{dom::{create_element}, model::tournament::{StageId, TournamentId, StageKind, FixtureId, FixtureTeam}, model::Model, ui::{UiElement, UiElementId, create_callback, EventList, Event}};

use super::create_callback_with_arg;

//TODO: drag fixture around

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
    selected_fixture_outputs: Vec<FixtureOutput>,
}

#[derive(PartialEq)]
struct FixtureOutput {
    fixture_id: FixtureId,
    outcome: Outcome,
}

#[derive(PartialEq)]
enum Outcome {
    Winner, Loser,
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
        dom_root.set_inner_text("Bracket view!");

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
            if let Some(UiElement::BracketView(this)) = ui.get_element(id) {
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
            current_drag: None, selected_fixture_outputs: vec![] };

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

                        let winner_handle: HtmlElement = new_div.query_selector("td[name=winner-handle]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let click_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_winner_handle_click(model, fid, e);
                            }
                        }));
                        winner_handle.set_onmousedown(Some(click_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(click_closure); // Needs to be kept alive

                        let loser_handle: HtmlElement = new_div.query_selector("td[name=loser-handle]").expect("Missing entry").expect("Missing entry").dyn_into().expect("Cast failed");
                        let click_closure = Box::new(create_callback_with_arg(move |model, ui, e| {
                            if let Some(UiElement::BracketView(this)) = ui.get_element_mut(id) {
                                this.on_fixture_loser_handle_click(model, fid, e);
                            }
                        }));
                        loser_handle.set_onmousedown(Some(click_closure.as_ref().as_ref().unchecked_ref()));
                        self.closures.push(click_closure); // Needs to be kept alive


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
                        for t in [&f.team_a, &f.team_b] {
                            match t {
                                FixtureTeam::Winner(f2) | FixtureTeam::Loser(f2) => {
                                    if let Some(f2) = fixtures.get(f2) {
                                        self.canvas_context.begin_path();
                                        self.canvas_context.move_to(f.layout.0 as f64, f.layout.1 as f64);
                                        self.canvas_context.line_to(f2.layout.0 as f64, f2.layout.1 as f64);
                                        self.canvas_context.stroke();
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_canvas_resize(&self, model: &Model) {
        self.canvas.set_width(self.canvas.client_width() as u32);
        self.canvas.set_height(self.canvas.client_height() as u32);

        self.redraw_canvas(model);
    }

    fn on_background_dblclick(&self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {

            let team_a = if self.selected_fixture_outputs.len() >= 1 {
                match self.selected_fixture_outputs[0] {
                    FixtureOutput { fixture_id, outcome: Outcome::Winner } => FixtureTeam::Winner(fixture_id),
                    FixtureOutput { fixture_id, outcome: Outcome::Loser } => FixtureTeam::Loser(fixture_id),
                }
            } else {
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

            let team_b = if self.selected_fixture_outputs.len() >= 2 {
                match self.selected_fixture_outputs[1] {
                    FixtureOutput { fixture_id, outcome: Outcome::Winner } => FixtureTeam::Winner(fixture_id),
                    FixtureOutput { fixture_id, outcome: Outcome::Loser } => FixtureTeam::Loser(fixture_id),
                }
            } else {
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

    fn on_fixture_drag_handle_mousedown(&mut self, model: &mut Model, fixture_id: FixtureId, e: MouseEvent) {
        if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
            let fixture_div_rect = fixture_div.get_bounding_client_rect();
            self.current_drag = Some(DragInfo { fixture_id, start_offset: (e.client_x() as f64 - fixture_div_rect.left(), e.client_y() as f64 - fixture_div_rect.top()) });
        }
    }

    fn on_background_mousemove(&self, model: &mut Model, e: MouseEvent) {
        if let Some(drag_info) = &self.current_drag {
            if let Some(fixture_div) = self.fixture_divs.get(&drag_info.fixture_id) {
                // Get the cursor position relative to the bracket view
                // This won't be the same as e.offsetX/Y if we are currently dragging over a child element of the bracket view,
                // as we receive the mousemove event via bubbling, so e.target will be the child element, and e.offsetX/Y will be relative to that instead.
              //  let e_target = e.target().and_then(|t| t.dyn_into::<HtmlElement>().ok());
              //  let target_rect =  e_target.as_ref().map_or(DomRect::new().unwrap(), |t| t.get_bounding_client_rect());
                let canvas_rect = self.canvas_container.get_bounding_client_rect();
                // let x = e.offset_x() as f64 + target_rect.left() - canvas_rect.left();
                // let y = e.offset_y() as f64 + target_rect.top() - canvas_rect.top();
                let x = e.client_x() as f64 - canvas_rect.left() - drag_info.start_offset.0;
                let y = e.client_y() as f64 - canvas_rect.top() - drag_info.start_offset.1;
                // debug!("target rect = {},{}. root rect = {},{}. updated to {x} {y}", target_rect.left(), target_rect.top(), canvas_rect.left(), canvas_rect.top());

                fixture_div.style().set_property("left", &x.to_string()).expect("Failed to set property");
                fixture_div.style().set_property("top", &y.to_string()).expect("Failed to set property");
            }
        }
    }

    fn on_background_mouseup(&mut self, model: &mut Model, e: MouseEvent) {
        if let (Some(tournament_id), Some(stage_id)) = (self.tournament_id, self.stage_id) {
            if let Some(drag_info) = &self.current_drag {
                let fixture_id = drag_info.fixture_id;
                if let Some(fixture_div) = self.fixture_divs.get(&fixture_id) {
                    let canvas_rect = self.canvas_container.get_bounding_client_rect();
                    let x = e.client_x() as f64 - canvas_rect.left() - drag_info.start_offset.0;
                    let y = e.client_y() as f64 - canvas_rect.top() - drag_info.start_offset.1;

                    self.current_drag = None;
                    if let Err(_) = model.set_fixture_layout(tournament_id, stage_id, fixture_id, (x as i32, y as i32)) {
                        error!("Failed to update fixture");
                    }
                }
            }
        }
    }

    fn on_fixture_winner_handle_click(&mut self, model: &mut Model, fixture_id: FixtureId, e: MouseEvent) {
        if !e.shift_key() {
            self.selected_fixture_outputs.clear();
            if let Ok(nodes) = self.canvas.query_selector_all("td[name=winner-handle],td[name=loser-handle]") {
                for i in 0..nodes.length() {
                    nodes.get(i).expect("Missing item").dyn_into::<HtmlElement>().expect("Failed cast").class_list().remove_1("selected").expect("Failed to remove class");
                }
            }
        }
        let x = FixtureOutput { fixture_id, outcome: Outcome::Winner };
        if !self.selected_fixture_outputs.contains(&x) {
            self.selected_fixture_outputs.push(x);
            if let Some(f) = self.fixture_divs.get(&fixture_id) {
                if let Ok(Some(x)) = f.query_selector("td[name=winner-handle]") {
                    x.dyn_into::<HtmlElement>().expect("Failed cast").class_list().add_1("selected").expect("Failed to add class");
                }
            }
        }
    }

    fn on_fixture_loser_handle_click(&mut self, model: &mut Model, fixture_id: FixtureId, e: MouseEvent) {
        if !e.shift_key() {
            self.selected_fixture_outputs.clear();
            if let Ok(nodes) = self.canvas.query_selector_all("td[name=winner-handle],td[name=loser-handle]") {
                for i in 0..nodes.length() {
                    nodes.get(i).expect("Missing item").dyn_into::<HtmlElement>().expect("Failed cast").class_list().remove_1("selected").expect("Failed to remove class");
                }
            }
        }
        let x = FixtureOutput { fixture_id, outcome: Outcome::Loser };
        if !self.selected_fixture_outputs.contains(&x) {
            self.selected_fixture_outputs.push(x);
            if let Some(f) = self.fixture_divs.get(&fixture_id) {
                if let Ok(Some(x)) = f.query_selector("td[name=loser-handle]") {
                    x.dyn_into::<HtmlElement>().expect("Failed cast").class_list().add_1("selected").expect("Failed to add class");
                }
            }
        }
    }
}
