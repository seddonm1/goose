---
title: Speech Extension
description: Add Speech MCP Server as a Goose Extension
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import YouTubeShortEmbed from '@site/src/components/YouTubeShortEmbed';

<YouTubeShortEmbed videoUrl="https://youtube.com/embed/rurAp_WzOiY" />


This tutorial covers how to add the [Speech MCP Server](https://github.com/Kvadratni/speech-mcp) as a Goose extension to enable real-time voice interaction, audio/video transcription, text-to-speech conversion, and multi-speaker audio generation.

:::info Requirement
[PortAudio](https://github.com/GoogleCloudPlatform/python-docs-samples/blob/main/scripts/readme-gen/templates/install_portaudio.tmpl.rst#install-portaudio) is required for PyAudio to capture audio from your microphone
:::

:::tip TLDR
<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>
  [Launch the installer](goose://extension?cmd=uvx&&arg=-p&arg=3.10.14&arg=speech-mcp@latest&id=speech_mcp&name=Speech%20Interface&description=Voice%20interaction%20with%20audio%20visualization%20for%20Goose)
  </TabItem>
  <TabItem value="cli" label="Goose CLI">
  **Command**
  ```sh
  uvx -p 3.10.14 speech-mcp@latest
  ```
  </TabItem>
</Tabs>
:::

## Configuration

:::info
Note that you'll need [uv](https://docs.astral.sh/uv/#installation) installed on your system to run this command, as it uses `uvx`.

Before adding this extension, make sure [PortAudio](https://github.com/GoogleCloudPlatform/python-docs-samples/blob/main/scripts/readme-gen/templates/install_portaudio.tmpl.rst#install-portaudio) is installed on your system. **PortAudio is required** for PyAudio to capture audio from your microphone.
:::

<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>
  1. [Launch the installer](goose://extension?cmd=uvx&&arg=-p&arg=3.10.14&arg=speech-mcp@latest&id=speech_mcp&name=Speech%20Interface&description=Voice%20interaction%20with%20audio%20visualization%20for%20Goose)
  2. Press `Yes` to confirm the installation
  3. Click `Save Configuration`
  4. Scroll to the top and click `Exit` from the upper left corner
  </TabItem>
  <TabItem value="cli" label="Goose CLI">
  1. Run the `configure` command:
  ```sh
  goose configure
  ```

  2. Choose to add a `Command-line Extension`
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

  3. Give your extension a name
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
    │  speech
    // highlight-end
    └ 
  ```

  4. Enter the command
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
    │  speech
    │
    // highlight-start
    ◆  What command should be run?
    │  uvx -p 3.10.14 speech-mcp@latest
    // highlight-end
    └ 
  ```  

  5. Enter the number of seconds Goose should wait for actions to complete before timing out. Default is 300s
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
    │  speech
    │
    ◇  What command should be run?
    │  uvx -p 3.10.14 speech-mcp@latest
    │
    // highlight-start
    ◆  Please set the timeout for this tool (in secs):
    │  300
    // highlight-end
    └ 
  ```  

  6. Choose to add a description. If you select "Yes" here, you will be prompted to enter a description for the extension.
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
    │  speech
    │
    ◇  What command should be run?
    │  uvx -p 3.10.14 speech-mcp@latest
    │
    ◇  Please set the timeout for this tool (in secs):
    │  300
    │
    // highlight-start
    ◇  Would you like to add a description?
    │  No
    // highlight-end
    └ 
  ```

  7. Choose No when asked to add environment variables
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
    │  speech
    │
    ◇  What command should be run?
    │  uvx -p 3.10.14 speech-mcp@latest
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
    │
    // highlight-end
    └  Added speech extension
  ```  

  </TabItem>
</Tabs>


## Example Usage
In this example, you'll see how to use Goose with the Speech MCP Server Extension to turn voice commands into code. By speaking directly to Goose, you can launch the speech interface, give a spoken prompt, and have Goose generate and open a webpage based on your request.

This allows you to build with Goose hands-free, making development more accessible and interactive.

<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>
   1. Open a new session in Goose Desktop
  </TabItem>
  <TabItem value="cli" label="Goose CLI">
  1. Open a terminal and start a new Goose session:

  ```sh
  goose session
  ```

  </TabItem>
</Tabs>

### Goose Prompt
Here are some things you can say to activate the speech interface:

- “Goose, I don’t feel like typing today.”
- “Let’s talk using voice.”
- “Can we have a voice conversation?”
- “I’d like to speak instead of typing.”
- “Goose, launch the speech UI.”

```
Goose I don't feel like talking today
```
### Goose Output
```
I understand! In that case, I can help you use voice commands instead. I'll launch the speech interface so you can speak your requests rather than type them

---- Launch Speech UI ----
```

### Goose Speech 🎙️

**Goose:** 
>_I've launched the speech interface. You can now speak your requests instead of typing. Just speak after you hear my voice and see the microphone activate. What would you like help with?_


**Max:** 

>_Goose, lets create a simple page with the word 'vibes' on it and maybe change colors and make it move around the page and then open this page in the browser once you're done_


<div style={{ width: "100%", height: 0, position: "relative", paddingBottom: "56.25%" }}>
  <iframe
    src="https://www.youtube.com/embed/vbD8IHwx-OY"
    frameBorder="0"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
    allowFullScreen
    title="YouTube video"
    style={{ position: "absolute", top: 0, left: 0, width: "100%", height: "100%" }}
  ></iframe>
</div>

