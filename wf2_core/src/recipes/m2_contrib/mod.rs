use crate::{recipes::m2::M2Recipe, recipes::m2::M2Templates};
use crate::{recipes::RecipeTemplate};

pub struct M2ContribRecipe;

///
/// This recipe is designed for those occasions when you want to contribute to the Magento 2 repository.
/// Follow the instructions here to fork/clone the Magento repo before running this recipe :
/// https://devdocs.magento.com/guides/v2.3/contributor-guide/contributing.html
///
impl M2ContribRecipe {
    pub fn new() -> M2Recipe {
        let mut recipe = M2Recipe::new();

        let mut templates = M2Templates::default();
        templates.unison = RecipeTemplate {
            bytes: include_bytes!("templates/sync.prf").to_vec()
        };

        {
            recipe.with_templates(templates);
        }

        recipe
    }
}
