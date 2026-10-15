(ns midnight-alley.static
  "Serves the browser build (index.html, client.wasm, JS, assets) from a folder."
  (:require [clojure.java.io :as io]
            [clojure.string :as str])
  (:import (java.io File)))

(def ^:private content-types
  {"html" "text/html; charset=utf-8"
   "js"   "text/javascript; charset=utf-8"
   "wasm" "application/wasm"
   "png"  "image/png"
   "svg"  "image/svg+xml"
   "css"  "text/css; charset=utf-8"
   "json" "application/json"
   "txt"  "text/plain; charset=utf-8"})

(defn- extension [^File f]
  (let [n (.getName f) i (.lastIndexOf n ".")]
    (when (pos? i) (str/lower-case (subs n (inc i))))))

(defn file-handler
  "Returns (fn [uri]) -> a ring response for a file under `root`, or nil.
  Anything that resolves outside `root` (../) is refused."
  [root]
  (let [root-dir (.getCanonicalFile (io/file root))
        prefix (str (.getPath root-dir) File/separator)]
    (fn [uri]
      (let [path (if (= uri "/") "/index.html" uri)
            f (.getCanonicalFile (io/file root-dir (subs path 1)))]
        (when (and (.startsWith (.getPath f) prefix) (.isFile f))
          {:status  200
           :headers {"Content-Type"  (get content-types (extension f) "application/octet-stream")
                     "Cache-Control" "no-cache"}
           :body    f})))))
