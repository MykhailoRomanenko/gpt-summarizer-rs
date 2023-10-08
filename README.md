# gpt-summarizer-rs

A CLI tool to scrap a web page and get a quick summary of its text.  
- Scraps the contents of a page
- Uses extractive summarization to retrieve the most important sentences (based on cosine similarity of all possible sentence pairs)
- Sends summarized text to ChatGPT API to retrieve a coherent summary

## Future improvements
- Smart scraping
- Multiple language support 
- Add TF-IDF as a second metric for extractive summarization
