---
title: "Top 5 MCP Servers I Use as a Developer with Goose"
description: "These 5 MCP servers help me automate my workflow and make me a better developer."
date: 2025-04-02
authors: 
    - adewale
---

![blog cover](mcp-servers-cover.png)

As a developer, finding the right tools that seamlessly work together can feel like discovering a superpower. And when you have a working process, it can sometimes be difficult to try out new tools.

With the introduction of MCPs, AI agents like Goose are able to plug in to my existing tools, and the only thing that changes with my workflow is that much welcomed automation that comes with it. I still do the same things I do, but backed by AI, I can now do them faster and with more confidence.

Today, I'm excited to share not just my favorite MCP servers, but the ones I actually use almost everyday with real applications that you can probably relate to as well.

<!--truncate-->

:::tip
You can ask Goose what you can do with an extension to get a list of all the features and example use cases you can try out. 
:::

## GitHub MCP Server: Everything GitHub

The [GitHub MCP Server](/docs/mcp/github-mcp) comes with quite a lot of functionality. It can help you create issues, pull requests, repositories, and branches. My most frequent use case for the GitHub MCP is reviewing and understanding pull requests.

For cases when it's a large pull request, or I don't understand what is going on, I can pass the PR to Goose, giving it the right context to make me understand and then act on the pull request. I'm even able to create a documentation update or changelog update from the file changes in the PR. This is definitely one of my favorite things. 

E.g 

```
Hey Goose, this pull request https://github.com/block/goose/pull/1949, has a lot of changes. Can you summarize into a changelog for me?
```

## Knowledge Graph Memory: Context on Steroids

The [Knowledge Graph Memory](/docs/mcp/knowledge-graph-mcp) extension is like giving Goose a photographic memory of your project or data. Like the name implies, it creates a graph of any information fed into it, connecting the dots between different pieces of information or as I like to use it for - documentation. 

If I'm working on a specific project or library and I don't want any hallucinations, I am able to feed Goose with the right context and it will be able to answer questions about the project or library with the right context.

This could be documentation of the project I'm currently working on, or even documentation of a library I'm using.

E.g

```
I'm currently in a project called Goose, read through the documentation in `documentation/docs/` folder and store key information in the knowledge graph. Use it for reference anytime I ask you about Goose.
```

## Fetch Extension: The Web in our Hands

I had a slightly hard time deciding between the [Tavily Web Search Extension](/docs/mcp/tavily-mcp) and The [Fetch Extension](/docs/mcp/fetch-mcp) because while I do use them both to access the web, the Fetch extension works more like default for me. With the example above using the Knowledge graph, I'm able to get information from the internet to give Goose additional context to work with. 

:::note
The Tavily Web Search Extension has deep research capabilities and is great for finding specific information, while the Fetch Extension is more about general web access and data retrieval.
:::

## Memory Extension: My Habits and Preferences

I use the [Memory Extension](/docs/mcp/memory-mcp) to remind Goose about my general preferences as I work - to default to JavaScript or Node when trying out new prototypes, if I prefer one naming convention or the other - maybe even how I like my coffee :D.

This works differently from the Knowledge Graph extension even though they both store information locally. When combined with the Knowledge Graph, it can also help maintain a clear trail of technical decisions and their rationale. For example I got stuck on a code migration and asked Goose to remember where we stopped, what we've tried so far, and what we want to do next for when I start a new session.


## VS Code Extension: Your Favorite Editor, Connected

One of the biggest points in conversations with people especially around vibe coding, is finding ways to track what changes are being made. While version control is always recommended, sometimes I want to be able to stop or change direction before going too far. The [VS Code Extension](/docs/mcp/vscode-mcp) alongside other features, allows me to preview the diff of my code changes before I commit them. 

I can choose to accept or refuse these changes, or tell Goose to try something else before any actual changes are made.


## The Power of Integration

As mentioned at the beginning of this post, the best thing about these MCP servers is how they plug into my existing workflow. I am able to:

- Start a new session on Goose which opens the current folder as a project in VS Code.
- Start work on any changes and get any context I need from either the Knowledge Graph or from the internet using the Fetch extension.
- Any attempts at making changes takes my preferences from the Memory extension into account.
- I can then review these changes right in VS Code and either accept or reject them.
- And complete the task by asking Goose to create a pull request for me. 

This is a simplified example of how I use these extensions together - I may not use all of them in every session, but having them available sure makes my workflow much smoother.

What are your favorite MCP servers? How do you use them together? Share your experiences with us on [Discord server](https://discord.gg/block-opensource)!

<head>
  <meta property="og:title" content="Top 5 MCP Servers I Use as a Developer with Goose Extensions" />
  <meta property="og:type" content="article" />
  <meta property="og:url" content="https://block.github.io/goose/blog/2025/04/01/top-5-mcp-servers" />
  <meta property="og:description" content="These 5 MCP servers help me automate my workflow and make me a better developer." />
  <meta property="og:image" content="http://block.github.io/goose/assets/images/mcp-servers-cover-6994acb4dec5a3b33d10ea61f7609e4b.png" />
  <meta name="twitter:card" content="summary_large_image" />
  <meta property="twitter:domain" content="block.github.io/goose" />
  <meta name="twitter:title" content="Top 5 MCP Servers I Use as a Developer with Goose Extensions" />
  <meta name="twitter:description" content="These 5 MCP servers help me automate my workflow and make me a better developer." />
  <meta name="twitter:image" content="http://block.github.io/goose/assets/images/mcp-servers-cover-6994acb4dec5a3b33d10ea61f7609e4b.png" />
</head>