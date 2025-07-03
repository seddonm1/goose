import type { Recipe } from "@site/src/components/recipe-card";

// Load all YAML files from your recipes folder
const recipeFiles = require.context(
  "@site/src/pages/recipes/data/recipes",
  false,
  /\.ya?ml$/
);

export function getRecipeById(id: string): Recipe | null {
  const allRecipes = loadAllRecipes();
  return allRecipes.find((recipe) => recipe.id === id) || null;
}

export async function searchRecipes(query: string): Promise<Recipe[]> {
  const allRecipes = loadAllRecipes();

  if (!query) return allRecipes;

  return allRecipes.filter((r) =>
    r.title?.toLowerCase().includes(query.toLowerCase()) ||
    r.description?.toLowerCase().includes(query.toLowerCase()) ||
    r.action?.toLowerCase().includes(query.toLowerCase()) ||
    r.activities?.some((a) => a.toLowerCase().includes(query.toLowerCase()))
  );
}

function loadAllRecipes(): Recipe[] {
  return recipeFiles.keys().map((key: string) => {
    const parsed = recipeFiles(key).default || recipeFiles(key);
    const id = key.replace(/^.*[\\/]/, "").replace(/\.(yaml|yml)$/, "");
    return normalizeRecipe({ ...parsed, id });
  });
}

function normalizeRecipe(recipe: any): Recipe {
  const cleaned: Recipe = {
    id: recipe.id || recipe.title?.toLowerCase().replace(/\s+/g, "-") || "untitled-recipe",
    title: recipe.title || "Untitled Recipe",
    description: recipe.description || "No description provided.",
    instructions: recipe.instructions,
    prompt: recipe.prompt,
    extensions: Array.isArray(recipe.extensions)
      ? recipe.extensions.map((ext: any) =>
          typeof ext === "string" ? { type: "builtin", name: ext } : ext
        )
      : [],
    activities: Array.isArray(recipe.activities) ? recipe.activities : [],
    version: recipe.version || "1.0.0",
    author:
      typeof recipe.author === "string"
        ? { contact: recipe.author }
        : recipe.author || undefined,
    action: recipe.action || undefined,
    persona: recipe.persona || undefined,
    tags: recipe.tags || [],
    recipeUrl: "",
    localPath: `documentation/src/pages/recipes/data/recipes/${recipe.id}.yaml`,
  };

  // Add parameters and populate missing required values
  if (Array.isArray(recipe.parameters)) {
    for (const param of recipe.parameters) {
      if (param.requirement === "required" && !param.value) {
        param.value = `{{${param.key}}}`;
      }
    }
    (cleaned as any).parameters = recipe.parameters;
  }

  const configForGoose = {
    title: cleaned.title,
    description: cleaned.description,
    instructions: cleaned.instructions,
    prompt: cleaned.prompt,
    activities: cleaned.activities,
    extensions: cleaned.extensions,
    parameters: (cleaned as any).parameters || []
  };
  
  const encoded = toBase64(JSON.stringify(configForGoose));
  cleaned.recipeUrl = `goose://recipe?config=${encoded}`;

  return cleaned;
}


function toBase64(str: string): string {
  if (typeof window !== "undefined" && window.btoa) {
    return window.btoa(unescape(encodeURIComponent(str)));
  }
  return Buffer.from(str).toString("base64");
}
