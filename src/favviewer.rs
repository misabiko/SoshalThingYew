use yew::prelude::*;

use crate::timeline::Timeline;

pub struct FavViewer {
    //link: ComponentLink<Self>,
    //boot_articles: Option<Vec<ArticleData>>
}

pub enum FavViewerMsg {
    //FetchedBootArticles(Vec<serde_json::Value>),
}

impl Component for FavViewer {
    type Message = FavViewerMsg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            //link,
            //boot_articles: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
			<div id="timelineContainer">
				<Timeline name="FavViewer"/>
			</div>
		}
    }
}