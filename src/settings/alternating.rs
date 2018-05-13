/// Describes the Background Alternating option for Splits Component.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Alternating {
	/// Only a sinlge Gradient
	Single,
	/// Alternating Gradients for every split
	Alternating
}
