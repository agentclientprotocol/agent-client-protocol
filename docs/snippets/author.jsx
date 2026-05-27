/**
 * Author byline for announcement and update posts.
 *
 * Renders a top border followed by a bold name (optionally linked to the
 * author's GitHub profile) and an optional role line beneath it.
 *
 * Usage:
 *
 *   import { Author } from "/snippets/author.jsx";
 *
 *   <Author
 *     name="Ben Brandt"
 *     role="Zed Industries / ACP Lead Maintainer"
 *     github="https://github.com/benbrandt"
 *   />
 */
export const Author = ({ name, role, github }) => (
  <div className="mt-8 border-t border-gray-200 pt-4 dark:border-gray-800">
    <div className="font-semibold">
      {github ? <a href={github}>{name}</a> : name}
    </div>
    {role ? <div className="mt-1 text-sm opacity-80">{role}</div> : null}
  </div>
);
