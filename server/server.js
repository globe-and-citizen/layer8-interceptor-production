// Using expressjs and multier create a server that can handle file uploads
// also one that can handle request params

const express = require('express');
const multer = require('multer');
const path = require('path');

// allow cross-origin requests
const cors = require('cors');
const { ppid } = require('process');
// Initialize the express application
const app = express();
app.use(cors()); // Enable CORS for all routes

const PORT = process.env.PORT || 3000;
// Set up multer for file uploads
const storage = multer.diskStorage({
    // Specify the destination and filename for uploaded files
    destination: (req, file, cb) => {
        cb(null, 'uploads/'); // Specify the directory to save uploaded files
    },
    filename: (req, file, cb) => {
        console.log('File received:', file);
        // Use the original file name 
        cb(null, file.originalname); // You can also use a unique name if needed
    }
});

const upload = multer({ storage: storage });

// Middleware to parse JSON bodies
app.use(express.json());
app.use(express.text()); // Middleware to parse text bodies

app.use((req, res, next) => {
    // Log the request method and URL
    console.log(`${req.method} request for '${req.url}'`);
    next(); // Call the next middleware or route handler
});

// Middleware to parse URL-encoded bodies
app.use(express.urlencoded({ extended: true }));


// Handle file uploads with a POST request
app.post('/formdata', upload.array('my_file'), (req, res) => {
    // Save my file to current directory
    if (!req.files) {
        return res.status(400).send('No file uploaded.');
    }

    return res.json({
        message: 'File uploaded successfully',
    });
});

// Handle request params with a GET request
app.get('/params', (req, res) => {
    const { message } = req.query; // Extract query parameters
    console.log('Received message:', message);
    res.send(`Received message: ${message}`);
});

app.post('/echo', (req, res) => {
    const body = req.body; // Extract message from the request body
    console.log('Received message:', body);
    res.send(body);
});

app.get('/hello', (req, res) => {
    res.send(`Hello, World!`); // Respond with a greeting
});

app.post('/hello', (req, res) => {

    console.log('Body received:', req.body); // Log the request body

    const name = req.body.name || 'World'; // Extract name from the request body
    res.send(`Hello, ${name}!`); // Respond with a greeting
});

// Start the server
app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});