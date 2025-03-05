import { 
    Connection, 
    PublicKey, 
    Keypair, 
    Transaction, 
    TransactionInstruction, 
    sendAndConfirmTransaction,
    SystemProgram
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { readFileSync } from 'fs';

// Connection configuration with commitment level
const connection = new Connection('https://api.devnet.solana.com', {
    commitment: 'confirmed',
    confirmTransactionInitialTimeout: 60000 // 60 seconds timeout
});

// Load keypair with error handling
let payer;
try {
    payer = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(readFileSync('./payer-keypair.json', 'utf8')))
    );
} catch (error) {
    console.error('Failed to load keypair:', error);
    process.exit(1);
}

// Constants and Public Keys
const PROGRAM_ID = new PublicKey('2ga161fxHesc8YATYz2CconNkTSpCJVABrjbBKGtRYGF');
const [rewardAccountPda, rewardBump] = await PublicKey.findProgramAddress(
    [Buffer.from("reward")],
    PROGRAM_ID
);
const ACCOUNTS = {
    reward: rewardAccountPda,
    userToken: new PublicKey('6UR1TvXTocdnjCWewwq7LiZfR9gnp8wS4R94pSsYhwja'),
    vaultToken: new PublicKey('3Jz4UFKq6NBke45J2en3UD733xpHkAekmW8Cn5Tsx4uA'),
    mint: new PublicKey('Bqw2nob1NpDCnEBEtPqnUVoDqW97JRUK8js5VjyC5Q4n'),
    tokenProgram: TOKEN_PROGRAM_ID
};

// Function to send transactions
async function sendTransaction(instructionData) {
    try {
        const transaction = new Transaction();
        const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
        
        transaction.feePayer = payer.publicKey;
        transaction.recentBlockhash = blockhash;
        transaction.lastValidBlockHeight = lastValidBlockHeight;
        transaction.add(instructionData);

        // Log instruction keys for debugging
        console.log('Instruction keys:');
        instructionData.keys.forEach((key, i) => {
            console.log(`Instruction key ${i}: ${key.pubkey.toBase58()} isSigner: ${key.isSigner}`);
        });
        
        // Sign transaction
        transaction.sign(payer);
        console.log('Transaction signatures:', transaction.signatures.map(s => s.publicKey.toBase58()));

        // Send and confirm with proper error handling and timeout
        const signature = await connection.sendRawTransaction(transaction.serialize(), {
            skipPreflight: false,
            preflightCommitment: 'confirmed',
        });

        const confirmation = await connection.confirmTransaction({
            signature,
            blockhash,
            lastValidBlockHeight,
        });

        if (confirmation.value.err) {
            throw new Error(`Transaction failed: ${confirmation.value.err}`);
        }

        console.log('Transaction confirmed:', signature);
        return signature;
    } catch (error) {
        console.error('Transaction failed:', error);
        throw error;
    }
}

// Initialize reward account
async function initRewardAccount() {
    try {
        const instructionData = Buffer.alloc(1);
        instructionData.writeUInt8(0, 0); // Init variant

        const instruction = new TransactionInstruction({
            programId: PROGRAM_ID,
            data: instructionData,
            keys: [
                { pubkey: payer.publicKey, isSigner: true, isWritable: true }, // Signer (payer)
                { pubkey: ACCOUNTS.reward, isSigner: false, isWritable: true }, // Reward account (PDA)
                { pubkey: ACCOUNTS.userToken, isSigner: false, isWritable: false },
                { pubkey: ACCOUNTS.vaultToken, isSigner: false, isWritable: false },
                { pubkey: ACCOUNTS.mint, isSigner: false, isWritable: false },
                { pubkey: ACCOUNTS.tokenProgram, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // System Program
            ],
        });

        return await sendTransaction(instruction);
    } catch (error) {
        console.error('Error initializing reward account:', error);
        throw error;
    }
}

// Earn points
async function earnPoints(points = 100) {
    try {
        const instructionData = Buffer.alloc(5);
        instructionData.writeUInt8(1, 0); // Earn variant
        instructionData.writeUInt32LE(points, 1);

        const instruction = new TransactionInstruction({
            programId: PROGRAM_ID,
            data: instructionData,
            keys: [
                { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                { pubkey: ACCOUNTS.reward, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.userToken, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.vaultToken, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.mint, isSigner: false, isWritable: false },
                { pubkey: ACCOUNTS.tokenProgram, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            ],
        });

        return await sendTransaction(instruction);
    } catch (error) {
        console.error('Error earning points:', error);
        throw error;
    }
}

// Claim reward
async function claimReward(requiredPoints = 50, amount = 2000n) {
    try {
        const instructionData = Buffer.alloc(13);
        instructionData.writeUInt8(2, 0); // Claim variant
        instructionData.writeUInt32LE(requiredPoints, 1);
        instructionData.writeBigUInt64LE(amount, 5);

        const instruction = new TransactionInstruction({
            programId: PROGRAM_ID,
            data: instructionData,
            keys: [
                { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                { pubkey: ACCOUNTS.reward, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.userToken, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.vaultToken, isSigner: false, isWritable: true },
                { pubkey: ACCOUNTS.mint, isSigner: false, isWritable: false },
                { pubkey: ACCOUNTS.tokenProgram, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
            ],
        });

        return await sendTransaction(instruction);
    } catch (error) {
        console.error('Error claiming reward:', error);
        throw error;
    }
}

// Check payer balance
async function checkPayerBalance() {
    try {
        const [balance, accountInfo] = await Promise.all([
            connection.getBalance(payer.publicKey),
            connection.getAccountInfo(payer.publicKey)
        ]);

        console.log(`Payer Public Key: ${payer.publicKey.toBase58()}`);
        console.log(`Balance: ${balance / 1e9} SOL`);
        console.log(`Account exists: ${accountInfo !== null}`);

        return { balance, accountInfo };
    } catch (error) {
        console.error('Error checking balance:', error);
        throw error;
    }
}

// Main execution
async function main() {
    try {
        console.log('Starting reward system interactions...');
        
        // Check balance before operations
        await checkPayerBalance();
        
        // Initialize the reward account (creation happens in the program)
        await initRewardAccount();
        
        // Execute operations
        await earnPoints();
        // await claimReward();
        
        console.log('Operations completed successfully');
    } catch (error) {
        console.error('Main execution failed:', error);
        process.exit(1);
    }
}

// Execute with proper error handling
if (import.meta.url === new URL(import.meta.url).href) {
    main().catch(console.error);
}

export {
    initRewardAccount,
    earnPoints,
    claimReward,
    checkPayerBalance
};
