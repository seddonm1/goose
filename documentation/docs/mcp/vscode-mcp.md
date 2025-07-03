---
title: VS Code Extension
description: Use the Goose VS Code Extension to connect Goose to your editor
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

This tutorial covers how to install and use the [Goose VS Code Extension](https://marketplace.visualstudio.com/items?itemName=block.vscode-goose) so you can use Goose interactively for file operations and development workflow management from within VS Code.

:::info Prerequisite
- [VS Code](https://code.visualstudio.com/) version 1.95.0 or higher is required
- [Goose Desktop](https://block.github.io/goose/) must be installed and running
:::

## Configuration

<Tabs>
<TabItem value="desktop" label="Goose Desktop">

1. Open VS Code
2. Open the Extensions view in VS Code:
   - Windows/Linux: `Ctrl+Shift+X`
   - macOS: `Cmd+Shift+X`
   - Or use: View → Extensions
3. Search for `VS Code Goose`
4. Click Install on [VS Code Goose](https://marketplace.visualstudio.com/items?itemName=block.vscode-goose) by Block
5. Restart VS Code if prompted
6. Open the Goose: Chat side panel to start a new conversation or view conversation history

</TabItem>
<TabItem value="cli" label="Goose CLI">

Not available via CLI.

</TabItem>
</Tabs>

:::tip
You can ask Goose about a particular file or code block by:
- Selecting the code you want to discuss
- Right-clicking and choosing "Ask Goose about this code"
- The code will appear as a reference chip above the chat input
:::

## Example Usage

The Goose VS Code Extension connects directly to your editor, letting you manage files, projects, and workflows with AI. It includes the following features:

- Interactive chat UI
- Access to Goose's AI capabilities directly within VS Code
- Unified session switching
- Code referencing with visual chips in the chat UI
- Quick actions for common coding tasks
- Clipboard tools to copy code snippets/responses for easy sharing


### Goose Prompt

```
Update the contributing guide at /goose/CONTRIBUTING.md with instructions on how to start docusaurus dev server
```

### Goose Output

```

I'll help you update the CONTRIBUTING.md file with instructions for starting the Docusaurus dev server. First, let me check the current content of the file.

─── developer__text_editor | view ──────────────────────────

I'll now update the CONTRIBUTING.md file to include instructions for starting the Docusaurus dev server. I'll add this information in a new section under the Node section.

─── developer__text_editor | str_replace ──────────────────────────

I've successfully updated the CONTRIBUTING.md file to include instructions for starting the Docusaurus dev server. I added a new "Documentation" section right after the Node section, which provides clear step-by-step instructions for:

Navigating to the docs directory
Installing dependencies
Starting the development server
I also included helpful information about the hot-reloading feature and the local server URL. The formatting is consistent with the rest of the document, using proper markdown syntax for code blocks and headers.

Is there anything else you'd like me to add or modify in the contributing guide?

```