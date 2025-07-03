---
title: Filesystem Extension
description: Add Filesystem MCP Server as Goose Extension
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import YouTubeShortEmbed from '@site/src/components/YouTubeShortEmbed';

<YouTubeShortEmbed videoUrl="https://youtube.com/embed/2IVPcjEr-yQ" /> 

This tutorial covers how to add the [Filesystem MCP server](https://github.com/modelcontextprotocol/servers/tree/HEAD/src/filesystem) as a Goose extension, enabling powerful code analysis and file management. With this extension, Goose can analyze project structures, edit and organize files, detect unused dependencies, and generate documentation to improve software maintainability.

:::tip TLDR
<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>
  [Launch the installer](goose://extension?cmd=npx&arg=-y&arg=@modelcontextprotocol/server-filesystem&arg=/path/to/allowed/directory&id=filesystem-mcp&name=Filesystem%20MCP%20Server&description=Filesystem%20MCP%20Server)
  </TabItem>
  <TabItem value="cli" label="Goose CLI">
  **Command**
  ```sh
  npx -y @modelcontextprotocol/server-filesystem </path/to/allowed/directory>
  ```
  </TabItem>
</Tabs>
    You can specify multiple allowed directories by separating them with a space.
:::

## Configuration

:::info
Note that you'll need [Node.js](https://nodejs.org/) installed on your system to run this commands, as is uses `npx`. 
:::

<Tabs groupId="interface">
  <TabItem value="ui" label="Goose Desktop" default>
    1. [Launch the installer](goose://extension?cmd=npx&arg=-y&arg=@modelcontextprotocol/server-filesystem&arg=/path/to/allowed/directory&id=filesystem-mcp&name=Filesystem%20MCP%20Server&description=Filesystem%20MCP%20Server)
    2. Press `Yes` to confirm the installation
    3. Add additional allowed directories if desired, separated by a space.
    4. Click `Save Configuration`
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
        │  filesystem
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
        │  filesystem
        │
        // highlight-start
        ◆  What command should be run?
        │  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/directory
        // highlight-end
        └ 
    ``` 
    :::tip Multiple Directories
    You can specify multiple allowed directories by separating them with a space.
    ::: 

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
        │  filesystem
        │
        ◇  What command should be run?
        │  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/directory
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
        │  filesystem
        │
        ◇  What command should be run?
        │  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/directory
        │
        ◇  Please set the timeout for this tool (in secs):
        │  300
        │
        // highlight-start
        ◆  Would you like to add a description?
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
        │  filesystem
        │
        ◇  What command should be run?
        │  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/directory
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
        └  Added filesystem extension
    ```  
  </TabItem>
</Tabs>

## Example Usage

In this example, I'll use Goose to perform a comprehensive security audit across multiple projects, including a payment processing system, authentication service, and e-commerce dashboard. I need to identify security vulnerabilities, implement fixes, and ensure sensitive data and operations are properly protected.

### Goose Prompt
    ```
    Hey Goose, I need to perform a security audit and implement fixes across multiple projects:

        1. Please analyze these codebases for security issues:
            - /Users/ebonyl/e-commerce-dashboard
            - /Users/ebonyl/client-projects/payment-system
            - /Users/ebonyl/internal/auth-service

        2. Look for:
            - Hardcoded credentials (API keys, passwords, secrets)
            - Insecure cryptographic practices (weak hashing, encryption)
            - SQL injection vulnerabilities
            - Exposed configuration files
            - Insecure session handling
            - Logging of sensitive data
            - Outdated dependencies in package.json
            - Unsafe authentication methods

        3. For each vulnerability found:
            - Show me the problematic code
            - Explain the security risk
            - Provide a secure implementation
            - Help me apply and test the fixes

        4. Create a detailed security report including:
            - Critical findings by project
            - Risk severity levels
            - Exact file locations and snippets
            - Applied fixes and improvements
            - Verification steps taken

    Please focus particularly on the payment system and authentication service as they handle sensitive data.
    ```
### Goose Output
    ```
    I'll help you conduct the security audit, starting with the payment system project since it handles sensitive financial data. Let me first check if we have access to these directories.

    --- List Allowed Directories ---

    Great, we have access to all the required directories. Let's start with the payment system project. First, I'll analyze its structure and files.

    --- Directory Tree ---

    Let's start by examining the key files for security issues. I'll first check the configuration and main payment processing files for potential vulnerabilities.

    --- Read Multiple Files ---

    I've identified several critical security vulnerabilities in the payment system. Let's address them one by one:

    1. Hardcoded Credentials (CRITICAL)

    Location: Both config/production.js and payment.js Risk: Exposed sensitive credentials in source code can lead to unauthorized access.
    ```
    ```JS
       // Current (Unsafe):
    const STRIPE_SECRET = 'sk_live_abcdef123456';
    const AWS_KEY = 'AKIA12345EXAMPLE';

    // Recommended Fix:
    const STRIPE_SECRET = process.env.STRIPE_SECRET_KEY;
    const AWS_KEY = process.env.AWS_ACCESS_KEY_ID;
    Let's create a secure environment configuration:

    ```
    ```
    Let's create a new secure version of the payment processor:

    --- Write File ---

    Let's also update the package.json to fix vulnerable dependencies:

    --- Read File ---

    Let's update the dependencies to their latest secure versions:

    --- Write File ---

    Security Report for Payment System
    Critical Findings
    
    1. Hardcoded Credentials (CRITICAL)
        - Multiple sensitive credentials in source code
        - Fixed by implementing environment variables
        - Added .env.example template
        
    2. Insecure Cryptographic Implementation (HIGH)
        - Weak encryption key
        - Improper key management
        - Fixed with proper AES-256-GCM implementation
    
    3. SSL/TLS Configuration (HIGH)
        - Disabled SSL verification
        - Fixed by enforcing TLS 1.2 and certificate validation
    
    4. Sensitive Data Exposure (HIGH)
        - Logging of sensitive card data
        - Fixed by removing sensitive logging
        - Implemented secure data encryption
    
    5. Race Conditions (MEDIUM)
        - Unsecured balance updates
        - Fixed with proper database transactions
    
    6. Vulnerable Dependencies (MEDIUM)
        - Outdated packages with known vulnerabilities
        - Updated all dependencies to latest secure versions
        - Added security audit npm script
    
    Verification Steps
        1. Create a proper .env file from the template
        2. Run npm install to update dependencies
        3. Run npm run security-audit to verify no known vulnerabilities
        4. Test the payment processing with test credentials
        5. Verify SSL/TLS configuration
        6. Test encryption/decryption functionality
        7. Verify secure balance updates with concurrent transactions

    I'll continue with the security audit of the remaining projects.

    --- Directory Tree ---
    
    Let's examine the key files for security issues. I'll start with the main application file and database configuration:

    --- Read Multiple Files ---

    Now, let's check the auth-service project:

    --- Directory Tree ---

    --- Read Multiple Files ---

    Based on the security audit of all three projects, I'll provide a comprehensive security report and implement fixes for each project.

    Comprehensive Security Audit Report

    1. Payment System Security Fixes (Already Implemented)

    ✅ Removed hardcoded credentials ✅ Implemented secure cryptography ✅ Fixed SSL/TLS configuration ✅ Removed sensitive data exposure ✅ Fixed race conditions ✅ Updated vulnerable dependencies

    2. E-commerce Dashboard Security Issues

    Let's create secure implementations for the e-commerce dashboard. First, let's create a .env template:

    --- Write File ---

    Now, let's update the main application with security fixes:

    --- Write File ---

    Let's update the database configuration:

    --- Write File ---

    3. Auth Service Security Issues

    Let's create secure implementations for the auth service. First, let's create a .env template:

    --- Write File ---

    Update the configuration:

    --- Write File ---

    Update the auth service with security fixes:

    --- Write File ---

    Update package.json with secure dependencies:

    --- Write File ---

    Final Security Report

    Critical Findings Summary

    1. Payment System 
        ✅ Credentials moved to environment variables
        ✅ Secure cryptography implemented
        ✅ SSL/TLS properly configured
        ✅ Sensitive data protection added
        ✅ Race conditions resolved
        ✅ Dependencies updated
    
    2. E-commerce Dashboard
        ✅ SQL injection vulnerabilities fixed
        ✅ Command injection risks mitigated
        ✅ Path traversal vulnerabilities resolved
        ✅ XSS protection implemented
        ✅ Secure session management added
        ✅ Rate limiting implemented
        ✅ CORS properly configured
        ✅ Error handling secured
        ✅ Database connection hardened
        ✅ Dependencies updated

    3. Auth Service
        ✅ Weak JWT secret replaced
        ✅ Password requirements strengthened
        ✅ Secure session management implemented
        ✅ Rate limiting added
        ✅ File upload security improved
        ✅ Input validation added
        ✅ Error handling secured
        ✅ Audit logging implemented
        ✅ Dependencies updated

    Verification Steps:
        1. Environment Setup:
    ```

    ```bash
    # For each project
    cp .env.example .env
    # Fill in secure values in .env files
    ```

    ```
        2. Update Dependencies:
    ```
    ```bash
    # In each project directory
    npm install
    npm audit
    ```
    ```
        3. Test Security Measures:
    ```
    ```bash
    # Rate limiting
    curl -X POST http://localhost:3000/api/login -d '{"username":"test","password":"test"}' -H 'Content-Type: application/json'

    # File upload
    curl -X POST http://localhost:3000/api/upload -F 'file=@test.jpg' -H 'Authorization: Bearer YOUR_TOKEN'

    # Protected routes
    curl http://localhost:3000/api/protected -H 'Authorization: Bearer YOUR_TOKEN'
    ```
    ```
        4. Security Headers Check:
    ```
    ```bash
    curl -I http://localhost:3000
    # Verify security headers are present
    ```
    ```
    Recommended Additional Steps

    1. Regular Security Audits:

        - Schedule monthly dependency updates
        - Run npm audit weekly
        - Monitor security advisories
    
    2. Monitoring:

        - Implement centralized logging
        - Set up alerts for suspicious activities
        - Monitor rate limit triggers
    
    3. Backup and Recovery:

        - Implement automated backups
        - Test recovery procedures
        - Document incident response
    
    4. Training:

        - Security awareness training for developers
        - Code review guidelines
        - Security best practices documentation
    
    Would you like me to help implement any additional security measures or explain any of the implemented fixes in more detail?
    ```
