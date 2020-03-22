use crate::offers::model::OfferEvent;

pub async fn offer_event(event: OfferEvent) {
  println!("{:?}", event);
}