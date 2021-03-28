export async function getKeplr() {
  if (window.keplr) {
    return window.keplr;
  } 
    
  if (document.readyState === "complete") {
    return window.keplr;
  } 
    
  return new Promise((resolve) => {
    const documentStateChange = (event) => {
      if (
        event.target &&
        event.target.readyState === "complete"
      ) {
        resolve(window.keplr);
        document.removeEventListener("readystatechange", documentStateChange);
      } 
    };
        
    document.addEventListener("readystatechange", documentStateChange);
  });
} 
