---
title: "How to Vibe Code Responsibly (with Goose)"
description: Vibe coding feels magical until it isn't. Learn how to flow with Goose while protecting your code, your team, and your future self.
authors: 
    - rizel
---

# How to Vibe Code Responsibly (with Goose)

![blog cover](responsible-vibe-code.png)

On Feb 2, 2025, Andrej Karpathy coined the phrase "[vibe coding](https://x.com/karpathy/status/1886192184808149383)". Vibe coding represents a new approach to coding where developers ask an AI agent to build something, and they go with the flow. 

The [Model Context Protocol (MCP)](https://modelcontextprotocol.io/introduction) makes this practice possible. Before MCP, developers copied and pasted context between applications. This workflow fell short of the promised AI agent automation that everyone claimed. Today, AI agents can work autonomously using MCP and integrate with any application, from GitHub to Cloudflare, YouTube, and Figma.

This shift democratizes coding. For example, it's empowered:

* Web developers to create video games with Unity  
* Designers and product managers to prototype full-stack applications  
* Business owners to transform their visions into functional products

It's a freeing experience. But too often, we're [Icarus](https://www.britannica.com/topic/Icarus-Greek-mythology) with the keyboard, vibe coding too close to the sun.

<!--truncate-->

## The Dark Side of Vibe Coding

This creative freedom comes with significant risks. Many developers have encountered serious issues while vibe coding:

* Committing code with security vulnerabilities  
* Introducing difficult-to-fix bugs on top of "spaghetti" code  
* Losing weeks or months of work due to lack of version control  
* Accidentally exposing sensitive information like environment variables and API keys in production

<blockquote className="twitter-tweet" data-dnt="true" align="center"><p lang="en" dir="ltr">Today was the worst day ever☹️<br />The project I had been working on for the last two weeks got corrupted, and everything was lost. Just like that, my SaaS was gone. Two weeks of hard work, completely ruined.<br />But!!!<br />I started from scratch and have already completed 50% of the work…</p>&mdash; CC Anuj (@vid_anuj) <a href="https://twitter.com/vid_anuj/status/1902379748501880934?ref_src=twsrc%5Etfw">March 19, 2025</a></blockquote>
<script async src="https://platform.twitter.com/widgets.js" charSet="utf-8"></script>


## A Better Way to Vibe Code with Goose

[Goose](https://block.github.io/goose) is an open source AI agent local to your machine with built-in features for safe vibe coding.  

:::note
Most folks define "vibe coding" as purely chaotic development with no rules. I'm redefining it as flowing with AI while protecting your project, team, and future self.
:::

### 1. Use `.gooseignore` to Protect Sensitive Files

Goose supports [`.gooseignore`](https://block.github.io/goose/docs/guides/using-gooseignore) files. The concept is similar to `.gitignore` files for your AI agent. It defines which files and folders Goose should *not* read, modify, or interact with.

Use this when you want to prevent:

* Accidental changes to environment variables  
* Modifications to sensitive configs  
* Changes to test fixtures or snapshots  
* Edits to infrastructure and deployment configs  
* Changes to code examples or documentation  
* Shell commands running in places they shouldn't

### 2. Create a plan

Goose's [`/plan`](https://block.github.io/goose/docs/guides/goose-cli-commands#examples) command helps you align with your agent before any code is touched, giving you a clear understanding of what it intends to do and how it will do it.

This is especially useful for tasks that span multiple files, involve side effects, or could impact critical areas of your codebase. No more guesswork—just a structured breakdown you can review and approve.

### 3. Choose the Right Mode for the Job

While letting your AI agent take the lead is fun, not every moment calls for full autonomy. Sometimes, you need to pause, review, or plan before any code changes. Goose offers several [modes](https://block.github.io/goose/docs/guides/goose-permissions) that help you stay in control without breaking your momentum. Here's how to use them intentionally during your sessions:

* **Chat Mode**  
  Goose will only respond with text so that you can brainstorm together.

* **Approval Mode**  
  Before Goose executes an action, it asks for your approval. This is helpful when you want to keep building fast but still want to know what's about to happen before it does.

* **Smart Approval**  
  In this mode, Goose requests your approval for risky actions. This mode is helpful for prototyping quickly while keeping guardrails in place.

* **Autonomous Mode**  
  In this mode, Goose moves forward without asking for approval. Using this mode is best if you feel confident in the direction and have safety nets in place.

### 4. Use Version Control Religiously

There are moments when AI agents change too many files and lines that the Control + Z can't fix. It's best to commit to every change that you or Goose make to get recovery points, clear diffs, and the ability to revert quickly.  

### 5. Ask Questions and Think Critically

Even if you're vibe coding, don't turn off your brain.

Ask Goose:

* Why did you make this change?  
* Is this secure?  
* How are we handling secrets?  
* Is this the best way to structure the database?

By pushing your agent to explain itself, you'll build a better product and learn more along the way.

### 6. Define .goosehints for Better Context

The [.goosehints](https://block.github.io/goose/docs/guides/using-goosehints) file gives Goose additional context about your project's coding standards, architectural preferences, and security practices.  

Here are a few examples:

* "Never expose API keys."  
* "Use prepared statements for database queries."  
* "Avoid using eval or unsafe dynamic code."

### 7. Integrate Goose into Your CI/CD

Before issues hit production, add [Goose to your CI/CD pipeline](/docs/tutorials/cicd) to:
- Automate code reviews  
- Validate documentation  
- Run security checks  

### 8. Use an Allowlist to Block Unsafe MCP Servers

Some MCP servers can introduce security risks, especially if compromised.

Use the Goose [allowlist](https://github.com/block/goose/blob/main/crates/goose-server/ALLOWLIST.md) feature to prevent Goose from calling unsafe or untrusted tools.

Here's how the team at Block is thinking about [securing the MCP](/blog/2025/03/31/securing-mcp).

### 9. Pick a High-Performing LLM

Not all LLMs are built the same. Goose plays best with:

* Claude Sonnet 3.5  
* GPT-4o  

Lower-performing models might work, but they're more likely to hallucinate or misunderstand your goals. Read more about how [different LLM's perform with Goose](https://block.github.io/goose/blog/2025/03/31/goose-benchmark/).  

## Watch Vibe Coding in Action  
Here’s how folks vibe code with Goose:  

<iframe width="560" height="315" src="https://www.youtube.com/embed/xZo3aA-vFi4?si=14bVczrCUwdKBZyg" title="The Great Great Off" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## Final Thoughts

Vibe coding isn't inherently wrong. It's marks a new chapter in how we build, and it opens the door for everyone. But experienced developers have a responsibility to define what smart, safe vibe coding looks like. Goose gives us the tools to set that standard, so the whole community can code creatively without sacrificing quality.

Download [Goose](https://block.github.io/goose/docs/getting-started/installation/), and start vibe coding with intention today!

<head>
  <meta property="og:title" content="How to Vibe Code Responsibly (with Goose)" />
  <meta property="og:type" content="article" />
  <meta property="og:url" content="https://block.github.io/goose/blog/2025/04/08/vibe-code-responsibly" />
  <meta property="og:description" content="Vibe coding feels magical until it isn't. Learn how to flow with Goose while protecting your code, your team, and your future self." />
  <meta property="og:image" content="http://block.github.io/goose/assets/images/responsible-vibe-code-a77f5e24a879edda943cc76f1fc0bd2a.png" />
  <meta name="twitter:card" content="summary_large_image" />
  <meta property="twitter:domain" content="block.github.io/goose" />
  <meta name="twitter:title" content="How to Vibe Code Responsibly (with Goose)" />
  <meta name="twitter:description" content="Vibe coding feels magical until it isn't. Learn how to flow with Goose while protecting your code, your team, and your future self." />
  <meta name="twitter:image" content="http://block.github.io/goose/assets/images/responsible-vibe-code-a77f5e24a879edda943cc76f1fc0bd2a.png" />
</head>