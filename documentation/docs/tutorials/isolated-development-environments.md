---
title: Isolated Development Environments
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

This guide shows you how to set up isolated development environments using the **[Container Use MCP](https://github.com/dagger/container-use)** with Goose. With this setup, your development work will be isolated to both git branches and containers, allowing you to experiment freely without affecting your main system state.
Note that the container-use MCP is very new and emerging, but offers powerful ways to do isolated development which are very agent friendly (build on tools like Docker, copy on write filesystems and more)

## Overview

The **[Container Use MCP](https://github.com/dagger/container-use)** server provides containerized development environments that integrate seamlessly with Goose. This allows you to:

- Work on changes isolated to git branches
- Run code in containers without affecting your local machine
- Easily reset and start fresh when needed
- Maintain clean separation between different projects and experiments
- Work on things in parallel

## Prerequisites

- Docker installed and running on your system
- Git installed and configured
- Goose installed and configured

## Installation

### Install Container Use

Head on over to the [Container Use README](https://github.com/dagger/container-use/blob/main/README.md) for up-to-date install instructions for this fast moving project.

## Adding to Goose

### Method 1: Quick Setup Link

Click this link to automatically add the extension to Goose:

**[Add Container-Use to Goose](goose://extension?cmd=cu&arg=stdio&id=container-use&name=container%20use&description=use%20containers%20with%20dagger%20and%20git%20for%20isolated%20environments)**

### Method 2: Manual Configuration

<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>

  1. Click `...` in the top right corner of the Goose Desktop.
  2. Select `Advanced Settings` from the menu.
  3. Under `Extensions`, click `Add custom extension`.
  4. Fill in the details:
     - **Type**: `Standard IO`
     - **ID**: `container-use`
     - **Name**: `Container Use`
     - **Description**: `Use containers with dagger and git for isolated environments`
     - **Command**: `cu`
     - **Arguments**: `stdio`
  5. Click `Add` button

  </TabItem>
  <TabItem value="cli" label="Goose CLI">

  1. Run the configuration command:
  ```bash
  goose configure
  ```

  2. Select `Add Extension` from the menu.

  3. Choose `Command-line Extension`.

  4. Follow the prompts:
     - **Extension name**: `Container Use`
     - **Command**: `cu stdio`
     - **Timeout**: `300` (or your preferred timeout)
     - **Environment variables**: None needed

  </TabItem>
  <TabItem value="config" label="Config File">

Add the following configuration to your `~/.config/goose/config.yaml` file:

```yaml
extensions:
  container-use:
    name: container-use
    type: stdio
    enabled: true
    cmd: cu
    args:
    - stdio
    envs: {}
    timeout: 300
```

  </TabItem>
</Tabs>

## Usage

Once the extension is enabled in Goose, you can:

### Starting Isolated Development

Simply mention in your conversation with Goose that you want to work in an isolated environment:

```
"I want to experiment with adding a new feature, but I want to do it in an isolated environment so I don't affect my main codebase."
```

Goose will automatically:
1. Create a new git branch for your work
2. Set up a containerized environment
3. Ensure all changes are isolated from your host system

### Working with Experiments

```
"Let me try a completely different approach to this algorithm. Can you set up an isolated environment where I can experiment?"
```

### Learning New Technologies

```
"I want to try out this new framework, but I don't want to install all its dependencies on my main system."
```

## Benefits

- **Safety**: Experiment without fear of breaking your main development environment
- **Reproducibility**: Consistent environments across different machines and team members
- **Isolation**: Multiple projects can run simultaneously without conflicts
- **Easy cleanup**: Remove containers and branches when done
- **Version control**: All changes are tracked in isolated git branches
- **Rollback capability**: Easily discard failed experiments

## Common Workflows

### Feature Development

1. Start a conversation with Goose about a new feature
2. Request isolated development environment
3. Goose creates branch and container
4. Develop and test the feature
5. If successful, merge the branch; if not, discard it

### Dependency Exploration

1. Ask Goose to explore a new library or tool
2. Work in isolated container with the dependency
3. Test compatibility and functionality
4. Decide whether to integrate into main project

### Refactoring

1. Request isolated environment for major refactoring
2. Make changes in safety of container and branch
3. Test thoroughly before merging
4. Easy rollback if issues arise

## Troubleshooting

### Common Issues

**Docker not running:**
- Ensure Docker Desktop is installed and running
- Check Docker daemon status: `docker info`

**Permission issues:**
- Ensure your user has permission to run Docker commands
- On Linux, add user to docker group: `sudo usermod -aG docker $USER`

**Git issues:**
- Ensure Git is properly configured with user name and email
- Check that you're in a Git repository when starting isolated work

### Getting Help

If you encounter issues:

1. Check the **[Container Use GitHub repository](https://github.com/dagger/container-use)** for documentation
2. Verify all prerequisites are installed and working
3. Join our [Discord community](https://discord.gg/block-opensource) for support

## Next Steps

With container-use enabled in Goose, you're ready to develop with confidence. Try starting a conversation about a project you've been hesitant to experiment with, and let Goose set up a safe, isolated environment for your exploration.

Remember: with isolated environments, there's no such thing as a failed experiment - only learning opportunities that don't affect your main codebase.
