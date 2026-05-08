async function aoRealtimeSearch(query) {
  const cacheKey = `ao_cache_${query.toLowerCase().replace(/\s/g, '_')}`;

  try {
    // Attempt Live Search (DuckDuckGo API)
    const response = await fetch(`https://duckduckgo.com{encodeURIComponent(query)}&format=json&no_html=1`);
    const data = await response.json();
    const result = data.AbstractText || "Information found, but no summary available.";

    // SAVE to local device memory
    localStorage.setItem(cacheKey, JSON.stringify({
      text: result,
      time: new Date().toLocaleString()
    }));

    return `[LIVE]: ${result}`;
  } catch (err) {
    // OFFLINE: Pull from local device memory
    const cached = localStorage.getItem(cacheKey);
    if (cached) {
      const memo = JSON.parse(cached);
      return `[OFFLINE - Saved ${memo.time}]: ${memo.text}`;
    }
    return "Offline: No saved data found for this topic.";
  }
}

