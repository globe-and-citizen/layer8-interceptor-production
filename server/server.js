// Using expressjs and multier create a server that can handle file uploads
// also one that can handle request params

const express = require('express');
const multer = require('multer');
const path = require('path');

// allow cross-origin requests
const cors = require('cors');
// Initialize the express application
const app = express();
app.use(cors()); // Enable CORS for all routes

const PORT = process.env.PORT || 3000;
// Set up multer for file uploads
const storage = multer.diskStorage({
    destination: (req, file, cb) => {
        cb(null, 'uploads/'); // Specify the directory to save uploaded files
    },
    filename: (req, file, cb) => {
        cb(null, Date.now() + path.extname(file.originalname)); // Use a unique filename
    }
});


const upload = multer({ storage: storage });

// Middleware to parse JSON bodies
app.use(express.json());
app.use(express.text()); // Middleware to parse text bodies
// app.use

// Middleware to parse URL-encoded bodies
app.use(express.urlencoded({ extended: true }));
// Serve static files from the 'uploads' directory
app.use('/uploads', express.static('uploads'));
// Handle file upload with a POST request
app.post('/upload', upload.single('file'), (req, res) => {
    if (!req.file) {
        return res.status(400).send('No file uploaded.');
    }
    res.json({
        message: 'File uploaded successfully',
        file: req.file
    });
});

// 
app.post('formdata', upload.single('file'), (req, res) => {
    // pick up the message from the form data
    const message = req.body.message || 'No message provided';
    console.log('Received message:', message);
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

// Start the server
app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});