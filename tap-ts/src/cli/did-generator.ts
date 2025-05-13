#!/usr/bin/env node

/**
 * Command line tool for DID generation
 * This tool provides functionality to generate DIDs with different key types and methods
 */

import { program } from 'commander';
import { createDIDKey, createDIDWeb, DIDKeyType } from '../index';
import inquirer from 'inquirer';
import chalk from 'chalk';
import fs from 'fs';
import path from 'path';

// Define the program
program
  .name('did-generator')
  .description('CLI tool for generating DIDs (Decentralized Identifiers)')
  .version('1.0.0');

// Add a command for interactive mode
program
  .command('interactive')
  .description('Start an interactive session to create a DID')
  .action(async () => {
    try {
      console.log(chalk.green('DID Generator Interactive Mode'));
      console.log(chalk.yellow('This tool will help you create a DID (Decentralized Identifier)'));
      
      // First, ask for the DID method
      const { didMethod } = await inquirer.prompt([
        {
          type: 'list',
          name: 'didMethod',
          message: 'Which DID method would you like to use?',
          choices: [
            { name: 'did:key - A cryptographic key-based DID', value: 'key' },
            { name: 'did:web - A web domain-based DID', value: 'web' }
          ]
        }
      ]);
      
      // Next, ask for the key type
      const { keyType } = await inquirer.prompt([
        {
          type: 'list',
          name: 'keyType',
          message: 'Which key type would you like to use?',
          choices: [
            { name: 'Ed25519 - Edwards-curve Digital Signature Algorithm', value: 'Ed25519' },
            { name: 'P-256 - NIST P-256 Elliptic Curve', value: 'P256' },
            { name: 'Secp256k1 - ECDSA with secp256k1 curve (used in Bitcoin)', value: 'Secp256k1' }
          ]
        }
      ]);
      
      let did;
      let didDocument;
      
      // If the user selected did:web, we need to ask for the domain
      if (didMethod === 'web') {
        const { domain } = await inquirer.prompt([
          {
            type: 'input',
            name: 'domain',
            message: 'Please enter the domain for the did:web:',
            validate: (input) => {
              // Basic domain validation (could be more comprehensive)
              if (/^[a-zA-Z0-9][-a-zA-Z0-9.]*[a-zA-Z0-9](\.[a-zA-Z0-9][-a-zA-Z0-9.]*[a-zA-Z0-9])+$/.test(input)) {
                return true;
              }
              return 'Please enter a valid domain name (e.g., example.com)';
            }
          }
        ]);
        
        // Create a did:web with the specified domain and key type
        const didKey = await createDIDWeb(domain, keyType as DIDKeyType);
        did = didKey.did;
        didDocument = didKey.didDocument;
        
        console.log(chalk.green(`\nCreated a new did:web identifier:`));
        console.log(chalk.blue(`DID: ${did}`));
        console.log(chalk.blue(`Key type: ${keyType}`));
        console.log(chalk.blue(`Public key: ${didKey.getPublicKeyHex()}`));
        console.log(chalk.blue(`DID Document:`));
        console.log(chalk.gray(JSON.stringify(JSON.parse(didDocument), null, 2)));
        
      } else {
        // Create a did:key with the specified key type
        const didKey = await createDIDKey(keyType as DIDKeyType);
        did = didKey.did;
        didDocument = didKey.didDocument;
        
        console.log(chalk.green(`\nCreated a new did:key identifier:`));
        console.log(chalk.blue(`DID: ${did}`));
        console.log(chalk.blue(`Key type: ${keyType}`));
        console.log(chalk.blue(`Public key: ${didKey.getPublicKeyHex()}`));
        console.log(chalk.blue(`DID Document:`));
        console.log(chalk.gray(JSON.stringify(JSON.parse(didDocument), null, 2)));
      }
      
      // Ask if the user wants to save the DID document
      const { save } = await inquirer.prompt([
        {
          type: 'confirm',
          name: 'save',
          message: 'Do you want to save the DID document to a file?',
          default: true
        }
      ]);
      
      if (save) {
        const { filepath } = await inquirer.prompt([
          {
            type: 'input',
            name: 'filepath',
            message: 'Enter the file path to save the DID document:',
            default: `./did-${did.replace(/[:]/g, '-').replace(/[^a-zA-Z0-9-_]/g, '_')}.json`
          }
        ]);
        
        // Ensure the directory exists
        const dirPath = path.dirname(filepath);
        if (!fs.existsSync(dirPath)) {
          fs.mkdirSync(dirPath, { recursive: true });
        }
        
        // Write the DID document to a file
        fs.writeFileSync(filepath, didDocument);
        console.log(chalk.green(`\nDID document saved to ${filepath}`));
      }
      
    } catch (error) {
      console.error(chalk.red('Error creating DID:'), error);
      process.exit(1);
    }
  });

// Add a command for direct creation of a did:key
program
  .command('key')
  .description('Create a did:key identifier')
  .option('-t, --type <type>', 'Key type (Ed25519, P256, or Secp256k1)', 'Ed25519')
  .option('-o, --output <file>', 'Output file for the DID document')
  .action(async (options) => {
    try {
      // Validate the key type
      const keyType = options.type;
      if (!['Ed25519', 'P256', 'Secp256k1'].includes(keyType)) {
        console.error(chalk.red('Invalid key type. Must be one of: Ed25519, P256, Secp256k1'));
        process.exit(1);
      }
      
      // Create a did:key with the specified key type
      const didKey = await createDIDKey(keyType as DIDKeyType);
      
      console.log(chalk.green(`Created a new did:key identifier:`));
      console.log(chalk.blue(`DID: ${didKey.did}`));
      console.log(chalk.blue(`Key type: ${keyType}`));
      console.log(chalk.blue(`Public key: ${didKey.getPublicKeyHex()}`));
      
      // Save the DID document if an output file was specified
      if (options.output) {
        const filepath = options.output;
        
        // Ensure the directory exists
        const dirPath = path.dirname(filepath);
        if (!fs.existsSync(dirPath)) {
          fs.mkdirSync(dirPath, { recursive: true });
        }
        
        // Write the DID document to a file
        fs.writeFileSync(filepath, didKey.didDocument);
        console.log(chalk.green(`DID document saved to ${filepath}`));
      }
      
    } catch (error) {
      console.error(chalk.red('Error creating DID:'), error);
      process.exit(1);
    }
  });

// Add a command for direct creation of a did:web
program
  .command('web')
  .description('Create a did:web identifier')
  .requiredOption('-d, --domain <domain>', 'Domain for the did:web (e.g., example.com)')
  .option('-t, --type <type>', 'Key type (Ed25519, P256, or Secp256k1)', 'Ed25519')
  .option('-o, --output <file>', 'Output file for the DID document')
  .action(async (options) => {
    try {
      // Validate the key type
      const keyType = options.type;
      if (!['Ed25519', 'P256', 'Secp256k1'].includes(keyType)) {
        console.error(chalk.red('Invalid key type. Must be one of: Ed25519, P256, Secp256k1'));
        process.exit(1);
      }
      
      // Create a did:web with the specified domain and key type
      const didKey = await createDIDWeb(options.domain, keyType as DIDKeyType);
      
      console.log(chalk.green(`Created a new did:web identifier:`));
      console.log(chalk.blue(`DID: ${didKey.did}`));
      console.log(chalk.blue(`Domain: ${options.domain}`));
      console.log(chalk.blue(`Key type: ${keyType}`));
      console.log(chalk.blue(`Public key: ${didKey.getPublicKeyHex()}`));
      
      // Save the DID document if an output file was specified
      if (options.output) {
        const filepath = options.output;
        
        // Ensure the directory exists
        const dirPath = path.dirname(filepath);
        if (!fs.existsSync(dirPath)) {
          fs.mkdirSync(dirPath, { recursive: true });
        }
        
        // Write the DID document to a file
        fs.writeFileSync(filepath, didKey.didDocument);
        console.log(chalk.green(`DID document saved to ${filepath}`));
      }
      
    } catch (error) {
      console.error(chalk.red('Error creating DID:'), error);
      process.exit(1);
    }
  });

// Parse the command line arguments
program.parse(process.argv);

// If no commands were provided, display the help
if (!process.argv.slice(2).length) {
  program.outputHelp();
}