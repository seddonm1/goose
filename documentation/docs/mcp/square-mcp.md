---
title: Square MCP Extension
description: Add the Square API as a Goose Extension
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import YouTubeShortEmbed from '@site/src/components/YouTubeShortEmbed';

<details>
  <summary> 🎥 Square MCP Server Video Walkthrough</summary>
  <iframe
  class="aspect-ratio"
  src="https://www.youtube.com/embed/y6pklrzhzNg"
  title="Run your Business with AI | Square MCP Server"
  frameBorder="0"
  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
  allowFullScreen
  ></iframe>
</details>


This tutorial will get you started with the [Square MCP server](https://developer.squareup.com/docs/mcp) to enable interactive and automated work for your Square seller account. The Square MCP server is an open source project that allows you to interact with the Square API using Goose.

Square offers two versions of the MCP server:

1. **Remote MCP server** hosted by Square, which uses OAuth for authentication and allows fine-grained permissions on API usage.
2. **Local MCP server** that you can run on your own machine, which uses an access token for authentication and allows full API access.

:::info
Note that you'll need [Node.js](https://nodejs.org/) installed on your system to run installation commands, which use `npx`.
:::


<Tabs groupId="remote-or-local">
  <TabItem value="remote" label="Square Remote MCP" default>
  :::tip TLDR
  <Tabs groupId="interface">
    <TabItem value="ui" label="Goose Desktop" default>
    [Launch the installer](https://mcp.squareup.com/goose)
    </TabItem>
    <TabItem value="cli" label="Goose CLI">
    **Command**
    ```sh
    npx mcp-remote https://mcp.squareup.com/sse
    ```
    </TabItem>
  </Tabs>
  :::

  ## Configuration

  <Tabs groupId="interface">
    <TabItem value="ui" label="Goose Desktop" default>
     1. [Launch the installer](https://mcp.squareup.com/goose)
     2. Goose will open and ask you to confirm installation.
     3. Goose should open a browser tab to an OAuth permissions page. Double-check which permissions you want to allow, and click 'Grant Access'.
     4. It will ask you to login or reauthenticate to Square, and may ask you to confirm the permissions you want to allow.
    </TabItem>
    <TabItem value="cli" label="Goose CLI">
  1. Run the `configure` command:
  ```sh
  goose configure
  ```

  1. Choose to add a `Command-line Extension`
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◆  What type of extension would you like to add?
    │  ○ Built-in Extension 
    │  ○ Command-line Extension (Run a local command or script)
    // highlight-start    
    │  ● Remote Extension 
    // highlight-end    
    └ 
  ```

  1. Give your extension a name
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Remote Extension 
    │
    // highlight-start
    ◆  What would you like to call this extension?
    │  square-mcp-remote
    // highlight-end
    └ 
  ```

  1. Enter the SSE URI
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Remote Extension
    │
    ◇  What would you like to call this extension?
    │  square-mcp-remote
    │
    // highlight-start
    ◆  What is the SSE endpoint URI?
    │  https://mcp.squareup.com/sse
    // highlight-end
    └ 
  ```  

  1. Enter the number of seconds Goose should wait for actions to complete before timing out. Default is 300s
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Remote Extension
    │
    ◇  What would you like to call this extension?
    │  square-mcp-remote
    │
    ◆  What is the SSE endpoint URI?
    │  https://mcp.squareup.com/sse
    │
    // highlight-start
    ◆  Please set the timeout for this tool (in secs):
    │  300
    // highlight-end
    └ 
  ```  

  1. Choose to add a description. If you select "Yes" here, you will be prompted to enter a description for the extension.
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Remote Extension
    │
    ◇  What would you like to call this extension?
    │  square-mcp-remote
    │
    ◆  What is the SSE endpoint URI?
    │  https://mcp.squareup.com/sse
    │
    ◆  Please set the timeout for this tool (in secs):
    │  300
    │
    // highlight-start
    ◇  Would you like to add a description?
    │  No
    // highlight-end
    └ 
  ```  

  1. Obtain a [Square Access Token](https://developer.squareup.com/apps) and paste it in.
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Remote Extension
    │
    ◇  What would you like to call this extension?
    │  square-mcp-remote
    │
    ◆  What is the SSE endpoint URI?
    │  https://mcp.squareup.com/sse
    │
    ◇  Please set the timeout for this tool (in secs):
    │  300
    │
    ◇  Would you like to add a description?
    │  No
    │
    // highlight-start
    ◆  Would you like to add environment variables?
    │  No
    // highlight-end
    │
    └  Added square-mcp-remote extension
  ```  
      </TabItem>
  </Tabs>


  </TabItem>



  <TabItem value="local" label="Square Local MCP">
  :::tip TLDR
  <Tabs groupId="interface">
    <TabItem value="ui" label="Goose Desktop" default>
    [Launch the installer](goose://extension?cmd=npx&arg=square-mcp-server%20start&id=square-mcp&name=Square%20MCP%20Server&description=Square%20MCP%20Server&env=ACCESS_TOKEN%3DYour%20Access%20Token&env=SANDBOX%3Dtrue)
    </TabItem>
    <TabItem value="cli" label="Goose CLI">
    **Command**
    ```sh
    npx square-mcp-server start
    ```
    </TabItem>
  </Tabs>
    **Environment Variables**
    ```
    ACCESS_TOKEN: <YOUR_API_KEY>
    SANDBOX: <true/false>
    PRODUCTION: <true/false>
    ```

    Note that you'll use `SANDBOX` -or- `PRODUCTION`, not both, and your `ACCESS_TOKEN` will either be a sandbox or production token, depending on which environment you choose.
  :::

  ## Configuration


  <Tabs groupId="interface">
    <TabItem value="ui" label="Goose Desktop" default>
  1. [Launch the installer](goose://extension?cmd=npx&arg=square-mcp-server%20start&id=square-mcp&name=Square%20MCP%20Server&description=Square%20MCP%20Server&env=ACCESS_TOKEN%3DYour%20Access%20Token&env=SANDBOX%3Dtrue)
  2. Press `Yes` to confirm the installation
  3. Get your [Square Access Token](https://developer.squareup.com/apps) and paste it in
  4. Keep `SANDBOX` as the environment variable, or change to `PRODUCTION`, and set its value to `true`
  5. Click `Save Configuration`
  6. Scroll to the top and click `Exit` from the upper left corner
  </TabItem>
  <TabItem value="cli" label="Goose CLI">

  1. Run the `configure` command:
  ```sh
  goose configure
  ```

  1. Choose to add a `Command-line Extension`
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◆  What type of extension would you like to add?
    │  ○ Built-in Extension 
    // highlight-start    
    │  ● Command-line Extension (Run a local command or script)
    // highlight-end    
    │  ○ Remote Extension 
    └ 
  ```

  1. Give your extension a name
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Command-line Extension 
    │
    // highlight-start
    ◆  What would you like to call this extension?
    │  square-mcp
    // highlight-end
    └ 
  ```

  1. Enter the command
  ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Command-line Extension 
    │
    ◇  What would you like to call this extension?
    │  square-mcp
    │
    // highlight-start
    ◆  What command should be run?
    │  npx square-mcp-server start
    // highlight-end
    └ 
  ```  

  1. Enter the number of seconds Goose should wait for actions to complete before timing out. Default is 300s
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Command-line Extension 
    │
    ◇  What would you like to call this extension?
    │  square-mcp
    │
    ◇  What command should be run?
    │  npx square-mcp-server start
    │
    // highlight-start
    ◆  Please set the timeout for this tool (in secs):
    │  300
    // highlight-end
    │
    └ 
  ```  

  1. Choose to add a description. If you select "Yes" here, you will be prompted to enter a description for the extension.
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Command-line Extension 
    │
    ◇  What would you like to call this extension?
    │  square-mcp
    │
    ◇  What command should be run?
    │  npx square-mcp-server start
    │
    ◆  Please set the timeout for this tool (in secs):
    │  300
    │
    // highlight-start
    ◇  Would you like to add a description?
    │  No
    // highlight-end
    │
    └ 
  ```  

  1. Obtain a [Square Access Token](https://developer.squareup.com/apps) and paste it in.
   ```sh
    ┌   goose-configure 
    │
    ◇  What would you like to configure?
    │  Add Extension (Connect to a new extension) 
    │
    ◇  What type of extension would you like to add?
    │  Command-line Extension 
    │
    ◇  What would you like to call this extension?
    │  square-mcp
    │
    ◇  What command should be run?
    │  npx square-mcp-server start
    │
    ◇  Please set the timeout for this tool (in secs):
    │  300
    │
    ◇  Would you like to add a description?
    │  No
    │
    // highlight-start
    ◆  Would you like to add environment variables?
    │  Yes 
    │
    ◇  Environment variable name:
    │  ACCESS_TOKEN
    │
    ◇  Environment variable value:
    │  ▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪▪
    │
    ◇  Add another environment variable?
    │  Yes
    │
    ◇  Environment variable name:
    │  SANDBOX
    │
    ◇  Environment variable value:
    │  true
    // highlight-end
    └  Added square-mcp extension
  ```  
    :::info 
    Change the `SANDBOX` key to `PRODUCTION` if using a production token
    :::

    </TabItem>
  </Tabs>

  </TabItem>
</Tabs>

## Example Usage

The Square MCP server allows you to interact with Square's connect API with an extensive [service catalog](https://github.com/square/square-mcp-server?tab=readme-ov-file#service-catalog) to access the Square API ecosystem.

#### Goose Prompt

```
( O)> Find my food menu from the following image and import these into my Square seller account. "my-burger-menu.png"
```

[Here's the menu image used in the demo.](../assets/guides/square-mcp-goosin-menu.png)

#### Goose Output

```
I'll help you import these menu items into your Square catalog. I'll use the Square API to create catalog items for each burger. Let me break this down into steps:

1. First, I'll check the catalog API service info and types
2. Then I'll create catalog items for each burger

Now let me check the type info for creating catalog objects

Now I'll start creating catalog items for each burger in your menu. I'll create them one by one using the catalog API. Let me process each burger:

(output truncated)

I've successfully created all the burger items from your menu in your Square catalog. Each burger has been added with:

- The exact name from your menu
- The full description
- The correct price
- A standard variation for ordering

Here's a summary of what was created:
- The Classic - $10.00
- Cheeseburger - $11.00
- Primetime Burger - $12.00
- Double Bacon - $14.00
- Texas Meatlover - $12.50
- Buffalo Burger - $12.00
- The Atomic - $12.50
- Little Bird - $11.00
- Mushroom Burger - $10.00

All items are now available in your Square catalog and ready for use in your point of sale system. Each burger includes its complete description and comes with a "Regular" variation at the specified price. The items are set as both sellable and stockable by default, and they're available at all locations.
```

