import {defineConfig} from 'vite';
export default defineConfig({server:{proxy:process.env.CITY_PREVIEW_TOKEN?{
 '/city-api':{target:'http://127.0.0.1:4783',headers:{'x-city-token':process.env.CITY_PREVIEW_TOKEN}}
}:undefined}});
